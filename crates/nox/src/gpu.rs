pub mod frame_graph;
mod pipeline;
mod image;
pub mod memory_binder;
pub mod linear_device_alloc;
mod context;

pub mod util;

mod host;
mod memory_layout;
mod handle;
mod helpers;
mod shader;
mod enums;
mod structs;
mod physical_device;
mod surface;
mod vulkan;
mod swapchain_context;
mod thread_context;
mod frame_context;
mod buffer;
mod global_resources;
mod commands;

use std::{
    sync::{Arc, RwLock},
};

pub use ash;

use ash::vk;

pub use vk::Format as VkFormat;

use winit::{
    window::Window,
    event_loop::ActiveEventLoop,
};

use compact_str::format_compact;

use nox_mem::{
    string_types::*,
    vec_types::{ArrayVec, FixedVec, GlobalVec, Vector},
};

use nox_alloc::arena_alloc::*;

use crate::dev::{
    export::*,
    utility::clamp,
    error::{self, Result, Error, Context, ErrorContext, Tracked, location},
    format_location,
};

use crate::win;

pub use host::*;
pub use context::GpuContext;
pub use enums::*;
pub use structs::*;
pub use memory_layout::MemoryLayout;
pub use handle::{Handle, RaiiHandle};
pub use image::*;
pub use buffer::*;
pub use physical_device::{PhysicalDeviceInfo, QueueFamilyIndices};
pub use global_resources::*;
pub use pipeline::*;
pub use commands::*;
pub use nox_proc::VertexInput;
pub use shader::*;
pub use pipeline::vertex_input::*;
pub use frame_graph::*;
use linear_device_alloc::LinearDeviceAlloc;

pub(crate) use surface::Surface;
pub(crate) use swapchain_context::{SwapchainContext, FrameData};
use vulkan::Vulkan;
use frame_context::{FrameContext, ResourcePool};
use swapchain_context::PresentResult;
use thread_context::ThreadContext;

pub type DeviceName = ArrayString<{ash::vk::MAX_PHYSICAL_DEVICE_NAME_SIZE}>;

pub const MIN_BUFFERED_FRAMES: u32 = 2;
pub const MAX_BUFFERED_FRAMES: u32 = 8;

#[derive(Clone, Copy)]
pub struct ComputeState {
    command_buffer: vk::CommandBuffer,
    semaphore: vk::Semaphore,
    timeline_value: u64,
}

pub(crate) struct Gpu {
    transfer_commands: GlobalVec<TransferCommandsStorage>,
    transfer_requests: TransferRequests,
    sync_transfer_semaphore: vk::Semaphore,
    sync_transfer_timeline_value: u64,
    sync_transfer_commands: GlobalVec<TransferCommandsStorage>,
    main_thread_context: ThreadContext,
    frame_resource_pools: ArrayVec<ResourcePool, {MAX_BUFFERED_FRAMES as usize}>,
    compute_states: ArrayVec<ComputeState, {MAX_BUFFERED_FRAMES as usize}>,
    graphics_command_buffers: ArrayVec<vk::CommandBuffer, {MAX_BUFFERED_FRAMES as usize}>,
    graphics_submit_fences: ArrayVec<vk::Fence, {MAX_BUFFERED_FRAMES as usize}>,
    global_resources: Arc<RwLock<GlobalResources>>,
    vk: Arc<Vulkan>,
    _memory_layout: MemoryLayout,
    buffered_frames: u32,
    current_frame_index: u32,
    tmp_alloc: Arc<ArenaAlloc>,
}

impl Gpu {

    pub fn new(
        event_loop: &ActiveEventLoop,
        app_name: &str,
        app_version: Version,
        enable_validation: bool,
        memory_layout: MemoryLayout,
        mut buffered_frames: u32,
        host_allocators: &HostAllocators,
    ) -> Result<Self>
    {
        buffered_frames = clamp(buffered_frames, MIN_BUFFERED_FRAMES, MAX_BUFFERED_FRAMES);
        assert!(buffered_frames <= MAX_BUFFERED_FRAMES);
        host_allocators
            .realloc_frame_graphs(buffered_frames)
            .context("failed to allocate frame graph host allocators")?;
        let tmp_alloc = Arc::new(
            ArenaAlloc
                ::new(memory_layout.tmp_arena_size())
                .context_with(|| format_location!("failed to create arena alloc at {loc}"))?
        );
        let vk = Arc::new(Vulkan
            ::new(
                event_loop,
                &app_name,
                app_version,
                enable_validation,
                &tmp_alloc
            ).context("failed to create vulkan backend")?);
        let main_thread_context = ThreadContext
            ::new(vk.clone())
            .context("failed to create main thread context")?;
        let global_resources = Arc::new(RwLock::new(
            GlobalResources
                ::new(vk.clone(), memory_layout)
                .context("failed to initialize global resources")?
        ));
        let mut s = Self {
            main_thread_context,
            vk: vk.clone(),
            frame_resource_pools: Default::default(),
            compute_states: Default::default(),
            graphics_command_buffers: Default::default(),
            graphics_submit_fences: Default::default(),
            global_resources: global_resources.clone(),
            _memory_layout: memory_layout,
            buffered_frames,
            current_frame_index: 0,
            transfer_commands: Default::default(),
            sync_transfer_semaphore: Default::default(),
            sync_transfer_timeline_value: 0,
            sync_transfer_commands: Default::default(),
            tmp_alloc,
            transfer_requests: Default::default(),
        };
        let mut i = 0;
        s.frame_resource_pools.try_resize_with(
            buffered_frames as usize,
            || {
                let device_alloc = LinearDeviceAlloc::new(
                    vk.clone(),
                    memory_layout.frame_graph_device_block_size(),
                    vk::MemoryPropertyFlags::DEVICE_LOCAL,
                    vk::MemoryPropertyFlags::LAZILY_ALLOCATED | vk::MemoryPropertyFlags::PROTECTED,
                    false,
                ).context_with(|| format_location!(
                    "failed to create linear device alloc at {loc}"
                ))?;
                let s = ResourcePool::new(device_alloc);
                i += 1;
                Ok(s)
            },
            |err| Error::new(ErrorContext::VecError(location!()),  err),
        )?;
        let mut compute_command_buffers
            = ArrayVec::<vk::CommandBuffer, {MAX_BUFFERED_FRAMES as usize}>::new();
        let mut compute_semaphores
            = ArrayVec::<vk::Semaphore, {MAX_BUFFERED_FRAMES as usize}>::new(); 
        unsafe {
            let mut timeline_info = vk::SemaphoreTypeCreateInfo {
                s_type: vk::StructureType::SEMAPHORE_TYPE_CREATE_INFO,
                semaphore_type: vk::SemaphoreType::TIMELINE,
                initial_value: 0,
                ..Default::default()
            };
            let semaphore_info = vk::SemaphoreCreateInfo {
                s_type: vk::StructureType::SEMAPHORE_CREATE_INFO,
                ..Default::default()
            }.push_next(&mut timeline_info);
            let alloc_info = vk::CommandBufferAllocateInfo {
                s_type: vk::StructureType::COMMAND_BUFFER_ALLOCATE_INFO,
                command_pool: s.main_thread_context.compute_pool(),
                level: vk::CommandBufferLevel::PRIMARY,
                command_buffer_count: buffered_frames,
                ..Default::default()
            };
            let result = (vk.device().fp_v1_0().allocate_command_buffers)(
                vk.device().handle(),
                &alloc_info,
                compute_command_buffers.as_mut_ptr(),
            );
            if result != vk::Result::SUCCESS {
                return Err(Error::new("failed to allocate compute command buffers", result))
            }
            compute_command_buffers.set_len(buffered_frames as usize);
            for _ in 0..buffered_frames {
                let fence = vk.device().create_semaphore(&semaphore_info, None).unwrap();
                compute_semaphores.push(fence).unwrap();
            }
            s.sync_transfer_semaphore = vk.device().create_semaphore(&semaphore_info, None).unwrap();
        }
        for (i, &buffer) in compute_command_buffers.iter().enumerate() {
            s.compute_states.push(ComputeState {
                command_buffer: buffer,
                semaphore: compute_semaphores[i],
                timeline_value: 0,
            }).unwrap();
        }
        unsafe {
            let alloc_info = vk::CommandBufferAllocateInfo {
                s_type: vk::StructureType::COMMAND_BUFFER_ALLOCATE_INFO,
                command_pool: s.main_thread_context.graphics_pool(),
                level: vk::CommandBufferLevel::PRIMARY,
                command_buffer_count: buffered_frames,
                ..Default::default()
            };
            let result = (vk.device().fp_v1_0().allocate_command_buffers)(
                vk.device().handle(),
                &alloc_info,
                s.graphics_command_buffers.as_mut_ptr(),
            );
            if result != vk::Result::SUCCESS {
                return Err(Error::new("failed to allocate graphics command buffers", result))
            }
            s.graphics_command_buffers.set_len(buffered_frames as usize);
        }
        let fence_create_info = vk::FenceCreateInfo {
            s_type: vk::StructureType::FENCE_CREATE_INFO,
            flags: vk::FenceCreateFlags::SIGNALED,
            ..Default::default()
        };
        s.graphics_submit_fences.try_resize_with(
            buffered_frames as usize,
            || unsafe {
                vk.device()
                .create_fence(&fence_create_info, None)
                .context("failed to create fence")
            },
            |err| Error::new("failed to create fences", err)
        )?;
        Ok(s)
    }

    #[inline(always)]
    pub fn context(&mut self) -> GpuContext<'_> {
        GpuContext::new(
            &self.vk,
            self.global_resources.write().unwrap(),
            &mut self.transfer_requests,
            self.buffered_frames
        )
    }

    #[inline(always)]
    pub fn wait_idle(&self) {
        unsafe {
            self.vk.device().device_wait_idle().ok();
        }
    }

    fn async_transfer_requests<'a>(
        &mut self,
        interface: &mut impl Interface,
    ) -> error::Result<()>
    {
        let count = self.transfer_requests.async_request_count();

        if count == 0 {
            return Ok(())
        }

        let device = self.vk.device();
        let queue_families = self.vk.queue_family_indices();
        let global_resources = self.global_resources.clone();

        let transfer_command_pool = Arc::new(TransientCommandPool
            ::new(self.vk.clone(), queue_families.transfer_index())
            .context("failed to create transient transfer command pool")?
        );
        let graphics_command_pool = Arc::new(TransientCommandPool
            ::new(self.vk.clone(), queue_families.graphics_index())
            .context("failed to create transient graphics command pool")?
        );

        let tmp_alloc = ArenaGuard::new(&self.tmp_alloc);

        let mut transfer_command_buffers = FixedVec
            ::with_len(count as usize, Default::default(), &tmp_alloc)
            .context(ErrorContext::VecError(location!()))?;

        let mut alloc_info = vk::CommandBufferAllocateInfo {
            s_type: vk::StructureType::COMMAND_BUFFER_ALLOCATE_INFO,
            command_pool: transfer_command_pool.handle(),
            level: vk::CommandBufferLevel::PRIMARY,
            command_buffer_count: count,
            ..Default::default()
        };

        helpers
            ::allocate_command_buffers(&device, &alloc_info, &mut transfer_command_buffers)
            .context("failed to allocate command buffers")?;

        let mut graphics_command_buffers = FixedVec
            ::with_len(count as usize, Default::default(), &tmp_alloc)
            .context(ErrorContext::VecError(location!()))?;

        alloc_info.command_pool = graphics_command_pool.handle();

        helpers
            ::allocate_command_buffers(&device, &alloc_info, &mut graphics_command_buffers)
            .context("failed to allocate command buffers")?;
        
        let mut new_requests = TransferRequests::default();

        for (i, (id, (staging_alloc, semaphores))) in self.transfer_requests.iter().enumerate() {

            let alloc = global_resources
                .write()
                .unwrap()
                .lock_linear_device_alloc(staging_alloc, semaphores)
                .context("failed to lock linear device alloc")?;

            helpers
                ::begin_command_buffer(&device, transfer_command_buffers[i])
                .context(ErrorContext::CommandBufferBeginError(location!()))?;
            helpers
                ::begin_command_buffer(&device, graphics_command_buffers[i])
                .context(ErrorContext::CommandBufferBeginError(location!()))?;

            let mut storage = TransferCommandsStorage::new(
                transfer_command_pool.clone(),
                transfer_command_buffers[i],
                graphics_command_pool.clone(),
                graphics_command_buffers[i],
                alloc,
                semaphores,
                id
            ).context("failed to initialize transfer commands")?;

            let mut gpu = GpuContext::new(
                &self.vk,
                global_resources.write().unwrap(),
                &mut new_requests,
                self.buffered_frames,
            );

            let mut commands = TransferCommands::new(&mut storage, &mut gpu);

            (interface)(Event::TransferWork {
                request_id: id,
                commands: &mut commands,
            }).context_from_tracked(|orig| ErrorContext::EventError(orig.or_this()))?;

            self.transfer_commands.push(storage);
        }

        self.transfer_requests.clear();

        if !new_requests.is_empty() {
            self.transfer_requests = new_requests;
        }

        Ok(())
    }

    fn process_transfer_requests<'a>(
        &mut self,
        transfer_queue: vk::Queue,
        graphics_queue: vk::Queue,
        pending_transfers: &mut GlobalVec<CommandRequestId>,
    ) -> error::Result<()>
    {
        if !self.transfer_commands.is_empty() {
            let mut ready_transfers = GlobalVec::with_capacity(self.transfer_commands.len());
            pending_transfers.reserve(pending_transfers.len() + self.transfer_commands.len());
            let mut dummy_requests = TransferRequests::default();
            for i in 0..self.transfer_commands.len()
            {
                let mut gpu = GpuContext::new(
                    &self.vk,
                    self.global_resources.write().unwrap(),
                    &mut dummy_requests,
                    self.buffered_frames,
                );
                let mut commands = TransferCommands::new(&mut self.transfer_commands[i], &mut gpu);
                let transfer_command_buffer = commands.transfer_command_buffer();
                let graphics_command_buffer = commands.graphics_command_buffer();
                let (new, sync_objects, signal_semaphores, context) = commands
                    .get_sync_objects()
                    .context("failed to create sync objects")?;
                if new {
                    let tmp_alloc = ArenaGuard::new(&self.tmp_alloc);
                    let mut signal_handles = FixedVec
                        ::with_capacity(signal_semaphores.len(), &tmp_alloc)
                        .context(ErrorContext::VecError(location!()))?;
                    let mut signal_values = FixedVec
                        ::with_capacity(signal_semaphores.len(), &tmp_alloc)
                        .context(ErrorContext::VecError(location!()))?;
                    for &(semaphore, value) in signal_semaphores {
                        let handle = context
                            .get_timeline_semaphore(semaphore)
                            .context("failed to find timeline semaphore")?;
                        signal_handles.push(handle).ok();
                        signal_values.push(value).ok();
                    }
                    unsafe {
                        self.vk.device()
                            .end_command_buffer(transfer_command_buffer)
                            .context("failed to end command buffer")?;
                    }
                    let submit_info = vk::SubmitInfo {
                        s_type: vk::StructureType::SUBMIT_INFO,
                        command_buffer_count: 1,
                        p_command_buffers: &transfer_command_buffer,
                        signal_semaphore_count: 1,
                        p_signal_semaphores: &sync_objects.binary_semaphore,
                        ..Default::default()
                    };
                    unsafe {
                        self.vk.device().queue_submit(
                            transfer_queue,
                            &[submit_info],
                            sync_objects.transfer_fence,
                        ).context(ErrorContext::TransferQueueSubmitError(location!()))?;
                    };
                    unsafe {
                        self.vk.device()
                            .end_command_buffer(graphics_command_buffer)
                            .context("failed to end command buffer")?;
                    }
                    let wait_stage = vk::PipelineStageFlags::TRANSFER;
                    let wait_value = 0;
                    let mut timeline_info = vk::TimelineSemaphoreSubmitInfo {
                        s_type: vk::StructureType::TIMELINE_SEMAPHORE_SUBMIT_INFO,
                        wait_semaphore_value_count: 1,
                        p_wait_semaphore_values: &wait_value,
                        signal_semaphore_value_count: signal_values.len() as u32,
                        p_signal_semaphore_values: signal_values.as_ptr(),
                        ..Default::default()
                    };
                    let submit_info = vk::SubmitInfo {
                        s_type: vk::StructureType::SUBMIT_INFO,
                        command_buffer_count: 1,
                        p_command_buffers: &graphics_command_buffer,
                        wait_semaphore_count: 1,
                        p_wait_semaphores: &sync_objects.binary_semaphore,
                        p_wait_dst_stage_mask: &wait_stage,
                        signal_semaphore_count: signal_handles.len() as u32,
                        p_signal_semaphores: signal_handles.as_ptr(),
                        ..Default::default()
                    }.push_next(&mut timeline_info);
                    unsafe {
                        self.vk.device().queue_submit(
                            graphics_queue,
                            &[submit_info],
                            sync_objects.graphics_fence,
                        ).context(ErrorContext::GraphicsQueueSubmitError(location!()))?;
                    }
                }
                unsafe {
                    let mut ready = false;
                    match self.vk.device().wait_for_fences(
                        &[sync_objects.transfer_fence, sync_objects.graphics_fence], true, 0
                    ) {
                        Ok(()) => {
                            ready = true;
                        },
                        Err(vk::Result::TIMEOUT) => {}
                        Err(err) => {
                            return Err(Error::new("failed to wait for fences", err))
                        }
                    }
                    if ready {
                        ready_transfers.push(i);
                    } else {
                        pending_transfers.push(commands.id());
                    }
                }
            }
            for i in ready_transfers.iter().rev() {
                self.transfer_commands.remove(*i);
            }
        }
        Ok(())
    }

    fn process_frame_graph<'a>(
        alloc: &'a ArenaAlloc,
        mut frame_graph: FrameGraphResult<'a>,
        graphics_queue: vk::Queue,
        transfer_queue: vk::Queue,
        sync_transfer_semaphore: vk::Semaphore,
        sync_transfer_timeline_value: &mut u64,
        sync_transfer_commands: &mut GlobalVec<TransferCommandsStorage>,
    ) -> Result<RenderResult<'a>>
    {
        let device = frame_graph.vk().device();
        let mut result = RenderResult::default();
        let swapchain_count = frame_graph.swapchain_count();
        if frame_graph.render_commands.transfer_commands.is_empty() {
            let count = frame_graph.wait_semaphore_count() as usize;
            result.wait_semaphores = FixedVec
                ::with_capacity(count + 1 + swapchain_count, alloc)
                .context_with(|| ErrorContext::VecError(location!()))?;
            result.wait_values = FixedVec
                ::with_capacity(count + 1 + swapchain_count, alloc)
                .context_with(|| ErrorContext::VecError(location!()))?;
            result.wait_stages = FixedVec
                ::with_capacity(count + 1 + swapchain_count, alloc)
                .context_with(|| ErrorContext::VecError(location!()))?;
        } else {
            let count = frame_graph.wait_semaphore_count() as usize;
            let mut transfer_command_buffers = FixedVec
                ::with_capacity(frame_graph.render_commands.transfer_commands.len(), alloc)
                .context_with(|| ErrorContext::VecError(location!()))?;
            let mut graphics_command_buffers = FixedVec
                ::with_capacity(frame_graph.render_commands.transfer_commands.len(), alloc)
                .context_with(|| ErrorContext::VecError(location!()))?;
            for storage in &frame_graph.render_commands.transfer_commands {
                let command_buffer = storage.transfer_command_buffer;
                unsafe {
                    device.end_command_buffer(command_buffer)
                        .context(ErrorContext::CommandBufferEndError(location!()))?
                }
                transfer_command_buffers.push(command_buffer).ok();
                let command_buffer = storage.graphics_command_buffer;
                unsafe {
                    device.end_command_buffer(command_buffer)
                        .context(ErrorContext::CommandBufferEndError(location!()))?
                }
                graphics_command_buffers.push(command_buffer).ok();
            }
            let fence_info = vk::FenceCreateInfo {
                s_type: vk::StructureType::FENCE_CREATE_INFO,
                ..Default::default()
            };
            let transfer_fence = unsafe {
                device.create_fence(&fence_info, None)
            }.context_with(|| format_compact!("failed to create fence at {}", location!()))?;
            *sync_transfer_timeline_value += 1;
            let mut timeline_info = vk::TimelineSemaphoreSubmitInfo {
                s_type: vk::StructureType::TIMELINE_SEMAPHORE_SUBMIT_INFO,
                signal_semaphore_value_count: 1,
                p_signal_semaphore_values: sync_transfer_timeline_value as *const _,
                ..Default::default()
            };
            let submit_info = vk::SubmitInfo {
                s_type: vk::StructureType::SUBMIT_INFO,
                command_buffer_count: transfer_command_buffers.len() as u32,
                p_command_buffers: transfer_command_buffers.as_ptr(),
                signal_semaphore_count: 1,
                p_signal_semaphores: &sync_transfer_semaphore,
                ..Default::default()
            }.push_next(&mut timeline_info);
            unsafe {
                device.queue_submit(
                    transfer_queue,
                    &[submit_info],
                    vk::Fence::null(),
                ).context(ErrorContext::TransferQueueSubmitError(location!()))?;
            };
            let wait_value = *sync_transfer_timeline_value;
            let wait_stage = vk::PipelineStageFlags::TRANSFER;
            *sync_transfer_timeline_value += 1;
            let mut timeline_info = vk::TimelineSemaphoreSubmitInfo {
                s_type: vk::StructureType::TIMELINE_SEMAPHORE_SUBMIT_INFO,
                wait_semaphore_value_count: 1,
                p_wait_semaphore_values: &wait_value,
                signal_semaphore_value_count: 1,
                p_signal_semaphore_values: sync_transfer_timeline_value,
                ..Default::default()
            };
            let submit_info = vk::SubmitInfo {
                s_type: vk::StructureType::SUBMIT_INFO,
                command_buffer_count: graphics_command_buffers.len() as u32,
                p_command_buffers: graphics_command_buffers.as_ptr(),
                wait_semaphore_count: 1,
                p_wait_semaphores: &sync_transfer_semaphore,
                p_wait_dst_stage_mask: &wait_stage,
                signal_semaphore_count: 1,
                p_signal_semaphores: &sync_transfer_semaphore,
                ..Default::default()
            }.push_next(&mut timeline_info);
            unsafe {
                device.queue_submit(
                    graphics_queue,
                    &[submit_info],
                    transfer_fence,
                ).context(ErrorContext::GraphicsQueueSubmitError(location!()))?;
            }
            result.wait_semaphores = FixedVec
                ::with_capacity(count + 2 + swapchain_count, alloc)
                .context_with(|| ErrorContext::VecError(location!()))?;
            result.wait_semaphores.push(sync_transfer_semaphore).ok();
            result.wait_values = FixedVec
                ::with_capacity(count + 2 + swapchain_count, alloc)
                .context_with(|| ErrorContext::VecError(location!()))?;
            result.wait_values.push(*sync_transfer_timeline_value).ok();
            result.wait_stages = FixedVec
                ::with_capacity(count + 2 + swapchain_count, alloc)
                .context_with(|| ErrorContext::VecError(location!()))?;
            result.wait_stages.push(vk::PipelineStageFlags::TRANSFER).ok();
        };
        sync_transfer_commands.move_from_vec(&mut frame_graph.render_commands.transfer_commands);
        let signal_count = frame_graph.signal_semaphore_count() as usize;
        result.signal_semaphores = FixedVec
            ::with_capacity(signal_count + 1 + swapchain_count, alloc)
            .context_with(|| ErrorContext::VecError(location!()))?;
        result.signal_values = FixedVec
            ::with_capacity(signal_count + 1 + swapchain_count, alloc)
            .context_with(|| ErrorContext::VecError(location!()))?;
        frame_graph.collect_semaphores(
            |frame_graph, id, value| {
                let handle = frame_graph.gpu().get_timeline_semaphore(id)?;
                result.signal_semaphores.push(handle).unwrap();
                result.signal_values.push(value).unwrap();
                Ok(())
            },
            |frame_graph, id, value, stage| {
                let handle = frame_graph.gpu().get_timeline_semaphore(id)?;
                result.wait_semaphores.push(handle).unwrap();
                result.wait_values.push(value).unwrap();
                result.wait_stages.push(stage.into()).unwrap();
                Ok(())
            }
        )?;
        result.frame_context = Some(frame_graph.frame_graph.finalize());
        Ok(result)
    }

    pub(crate) fn render<'a>(
        &mut self,
        win: &mut win::WindowContext<'_, 'a>,
        interface: &mut impl Interface,
        host_allocators: &'a HostAllocators,
        tmp_alloc: ArenaGuard,
    ) -> error::Result<()>
    {
        let graphics_queue = self.vk.graphics_queue();
        let transfer_queue = self.vk.transfer_queue();
        let compute_queue = self.vk.compute_queue();
        self.async_transfer_requests(interface)
            .context("async transfer requests failed")?;
        let mut pending_transfers = GlobalVec::new();
        self.process_transfer_requests(
            transfer_queue,
            graphics_queue,
            &mut pending_transfers,
            ).context("failed to process transfer requests")?;
        self.global_resources
            .write()
            .unwrap()
            .update_semaphores()
            .context("failed to update semaphores")?;
        let frame_index = self.current_frame_index as usize;
        let queue_family_indices = self.vk.queue_family_indices();
        let compute_state = self.compute_states[frame_index];
        unsafe {
            self.vk.device().reset_command_buffer(
                compute_state.command_buffer, vk::CommandBufferResetFlags::RELEASE_RESOURCES
            ).unwrap();
        } 
        {
            helpers::begin_command_buffer(self.vk.device(), compute_state.command_buffer).unwrap();
            let mut compute_commands = ComputeCommands::new(
                compute_state.command_buffer,
                GpuContext::new(
                    &self.vk,
                    self.global_resources.write().unwrap(),
                    &mut self.transfer_requests,
                    self.buffered_frames,
                ),
                &self.tmp_alloc,
                queue_family_indices.compute_index(),
            );
            (interface)(Event::ComputeWork {
                commands: &mut compute_commands
            }).context_from_tracked(|orig| ErrorContext::EventError(orig.or_this()))?;
            let compute_commands = compute_commands.finish();
            unsafe {
                self.vk.device().end_command_buffer(compute_state.command_buffer).unwrap();
            }
            let wait_count = 1 + compute_commands.wait_semaphores.len();
            let signal_count = 1 + compute_commands.signal_semaphores.len();
            let tmp_alloc = ArenaGuard::new(&self.tmp_alloc);
            let mut wait_handles = FixedVec
                ::with_capacity(wait_count, &tmp_alloc)
                .context(ErrorContext::VecError(location!()))?;
            let mut wait_values = FixedVec
                ::with_capacity(wait_count, &tmp_alloc)
                .context(ErrorContext::VecError(location!()))?;
            let mut wait_stages = FixedVec
                ::with_capacity(wait_count, &tmp_alloc)
                .context(ErrorContext::VecError(location!()))?;
            let mut signal_handles = FixedVec
                ::with_capacity(signal_count, &tmp_alloc)
                .context(ErrorContext::VecError(location!()))?;
            let mut signal_values = FixedVec
                ::with_capacity(signal_count, &tmp_alloc)
                .context(ErrorContext::VecError(location!()))?;
            let g = self.global_resources.read().unwrap();
            for &(id, value, stage) in &compute_commands.wait_semaphores {
                let handle = g.get_timeline_semaphore(id)?;
                wait_handles.push(handle).ok();
                wait_values.push(value).ok();
                wait_stages.push(stage.into()).ok();
            }
            for &(id, value) in &compute_commands.signal_semaphores {
                let handle = g.get_timeline_semaphore(id)?;
                signal_handles.push(handle).ok();
                signal_values.push(value).ok();
            }
            wait_handles.push(compute_state.semaphore).ok();
            wait_values.push(compute_state.timeline_value).ok();
            wait_stages.push(vk::PipelineStageFlags::TOP_OF_PIPE).ok();
            signal_handles.push(compute_state.semaphore).ok();
            signal_values.push(compute_state.timeline_value + 1).ok();
            let wait_count = wait_count as u32;
            let signal_count = signal_count as u32;
            let mut timeline_submit = vk::TimelineSemaphoreSubmitInfo {
                s_type: vk::StructureType::TIMELINE_SEMAPHORE_SUBMIT_INFO,
                wait_semaphore_value_count: wait_count,
                p_wait_semaphore_values: wait_values.as_ptr(),
                signal_semaphore_value_count: signal_count,
                p_signal_semaphore_values: signal_values.as_ptr(),
                ..Default::default()
            };
            let compute_submit = vk::SubmitInfo {
                s_type: vk::StructureType::SUBMIT_INFO,
                command_buffer_count: 1,
                p_command_buffers: &compute_state.command_buffer,
                wait_semaphore_count: wait_count,
                p_wait_semaphores: wait_handles.as_ptr(),
                p_wait_dst_stage_mask: wait_stages.as_ptr(),
                signal_semaphore_count: signal_count,
                p_signal_semaphores: signal_handles.as_ptr(),
                ..Default::default()
            }.push_next(&mut timeline_submit);
            unsafe {
                self.vk.device().queue_submit(
                    compute_queue,
                    &[compute_submit],
                    Default::default(),
                ).context(ErrorContext::ComputeQueueSubmitError(location!()))?;
            }
        }
        let graphics_command_buffer = self.graphics_command_buffers[frame_index];
        helpers
            ::begin_command_buffer(self.vk.device(), graphics_command_buffer)
            .context_with(|| format_compact!("failed to begin command buffer at {}", location!()))?;
        let graphics_submit_fence = self.graphics_submit_fences[frame_index];
        unsafe {
            self.vk.device().wait_for_fences(
                &[graphics_submit_fence],
                true,
                SwapchainContext::frame_timeout(),
            ).context("failed to wait for graphics submit fence")?;
            self.vk
                .device()
                .reset_fences(&[graphics_submit_fence])
                .context("failed to reset graphics submit fence")?;
        }
        let mut surfaces = FixedVec::with_capacity(
            win.active_ids().len(),
            &tmp_alloc,
        ).context(ErrorContext::VecError(location!()))?;
        for (id, win) in win.window_iter_mut() {
            let id = *id;
            let surface = win.surface();
            let (swapchain, recreated) = surface
                .get_or_init_swapchain_context(
                    host_allocators,
                )?;
            let frame_data = match swapchain.setup_image(
                self.vk.swapchain_device(),
                frame_index,
            )?
            {
                Some(r) => r,
                None => continue,
            };
            if recreated {
                let frame_buffer_size = frame_data.extent.into();
                {
                    let mut context = GpuContext::new(
                        &self.vk,
                        self.global_resources.write().unwrap(),
                        &mut self.transfer_requests,
                        self.buffered_frames,
                    );
                    (interface)(Event::FrameBufferCreated {
                        window_id: id, 
                        gpu: &mut context,
                        new_size: frame_buffer_size,
                        new_format: ImageFormat(frame_data.format, vk::ImageAspectFlags::COLOR),
                    }).context_from_tracked(|orig| ErrorContext::EventError(orig.or_this()))?;
                }
                self.async_transfer_requests(interface)
                    .context("async transfer requests failed")?;
                self.process_transfer_requests(transfer_queue, graphics_queue, &mut pending_transfers)
                    .context("failed to process transfer requests")?;
            }
            let present_queue = surface.present_queue();
            win.update_frame_data(frame_data);
            surfaces
                .push((win, id, present_queue, frame_data.suboptimal))
                .unwrap();
        }
        let alloc = &host_allocators.frame_graphs()[frame_index];
        unsafe {
            alloc.force_clear();
        }
        self.sync_transfer_commands.clear();
        let RenderResult {
            mut frame_context,
            mut wait_semaphores, mut wait_values, mut wait_stages,
            mut signal_semaphores, mut signal_values,
            transfer_fence
        } =
        {
            let mut frame_graph = FrameGraph::new(
                FrameContext::new(
                    graphics_command_buffer,
                    GpuContext::new(
                        &self.vk,
                        self.global_resources
                            .write()
                            .unwrap(),
                        &mut self.transfer_requests,
                        self.buffered_frames,
                    ),
                    &mut self.frame_resource_pools[frame_index],
                    surfaces.iter().map(|(win, id, _, _)| {
                        (*id, win.last_frame_data())
                    }),
                ),
                graphics_command_buffer,
                alloc,
                frame_index,
                queue_family_indices,
            );
            (interface)(Event::Render {
                frame_graph: &mut frame_graph,
                pending_transfers: &pending_transfers,
            }).context_from_tracked(|orig| ErrorContext::EventError(orig.or_this()))?;
            let frame_graph = frame_graph.render(
                interface,
                compute_state.semaphore,
                compute_state.timeline_value,
                self.buffered_frames,
            )?;
            Self::process_frame_graph(
                alloc,
                frame_graph,
                graphics_queue,
                transfer_queue,
                self.sync_transfer_semaphore,
                &mut self.sync_transfer_timeline_value,
                &mut self.sync_transfer_commands,
            )?
        };
        let mut frame_context = frame_context.take().unwrap();
        wait_semaphores.push(compute_state.semaphore).unwrap();
        wait_values.push(compute_state.timeline_value + 1).unwrap();
        wait_stages.push(vk::PipelineStageFlags::COMPUTE_SHADER).unwrap();
        signal_semaphores.push(compute_state.semaphore).unwrap();
        signal_values.push(compute_state.timeline_value + 2).unwrap();
        for (win, id, _, _) in &mut surfaces {
            let id = *id;
            let fallback_state = win
                .last_frame_data().image_state;
            let semaphores = win
                .surface()
                .get_swapchain_context()
                .unwrap()
                .setup_submit(
                    self.vk.device(),
                    graphics_command_buffer,
                    frame_context
                        .swapchain_image_state(id)
                        .cloned()
                        .unwrap_or(fallback_state),
                    queue_family_indices.graphics_index(),
                    frame_index,
                );
            wait_semaphores.push(semaphores.wait_semaphore).unwrap();
            wait_values.push(0).unwrap();
            wait_stages.push(semaphores.wait_stage).unwrap();
            signal_semaphores.push(semaphores.signal_semaphore).unwrap();
            signal_values.push(0).unwrap();
        }
        unsafe { self.vk.device()
            .end_command_buffer(graphics_command_buffer)
            .context(ErrorContext::CommandBufferEndError(location!()))?;
        }
        let wait_count = wait_semaphores.len() as u32;
        let signal_count = signal_semaphores.len() as u32;
        let mut timeline_submit = vk::TimelineSemaphoreSubmitInfo {
            s_type: vk::StructureType::TIMELINE_SEMAPHORE_SUBMIT_INFO,
            wait_semaphore_value_count: wait_count,
            p_wait_semaphore_values: wait_values.as_ptr(),
            signal_semaphore_value_count: signal_count,
            p_signal_semaphore_values: signal_values.as_ptr(),
            ..Default::default()
        };
        let mut submit_info = vk::SubmitInfo {
            s_type: vk::StructureType::SUBMIT_INFO,
            wait_semaphore_count: wait_count,
            p_wait_semaphores: wait_semaphores.as_ptr(),
            p_wait_dst_stage_mask: wait_stages.as_ptr(),
            signal_semaphore_count: signal_count,
            p_signal_semaphores: signal_semaphores.as_ptr(),
            command_buffer_count: 1,
            p_command_buffers: &graphics_command_buffer,
            ..Default::default()
        };
        submit_info = submit_info.push_next(&mut timeline_submit);
        if let Err(err) = unsafe { self.vk
            .device()
            .queue_submit(graphics_queue, &[submit_info], graphics_submit_fence)
        } {
            return Err(Error::new(ErrorContext::GraphicsQueueSubmitError(location!()), err))
        }
        if !self.sync_transfer_commands.is_empty() {
            unsafe {
                self.vk.device()
                    .wait_for_fences(&[transfer_fence], true, u64::MAX)
                    .context_with(|| format_compact!("failed to wait for fences at {}", location!()))?;
                self.vk.device()
                    .destroy_fence(transfer_fence, None);
            }
        }
        let mut request_swapchain_update = FixedVec
            ::with_capacity(surfaces.len(), &tmp_alloc)
            .context(ErrorContext::VecError(location!()))?;
        for (i, (win, id, present_queue, suboptimal)) in surfaces.iter_mut().enumerate() {
            let present_result = win
                .surface()
                .get_swapchain_context()
                .unwrap()
                .present_submit(self.vk.swapchain_device(), *present_queue)
                .context_with(|| format_compact!(
                    "failed to present to window (id {id:?})",
                ))?;
            if present_result != PresentResult::Success || *suboptimal {
                request_swapchain_update.push(i).unwrap();
            }
        }
        for &idx in &request_swapchain_update {
            let (win, _, _, _) = &mut surfaces[idx];
            let size = win.size().into();
            win.surface().request_swapchain_update(
                self.buffered_frames,
                size,
            );
        }
        self.compute_states[frame_index].timeline_value += 2;
        self.current_frame_index = (self.current_frame_index + 1) % self.buffered_frames;
        Ok(())
    }

    pub(crate) fn clean_up<'a>(&mut self) {
        log::info!("terminating renderer");
        unsafe {
            self.vk.device().device_wait_idle().ok();
        }
        for pool in &mut self.frame_resource_pools {
            unsafe {
                pool.force_clean_up();
            }
        }
        for state in &self.compute_states {
            unsafe {
                self.vk.device().destroy_semaphore(
                    state.semaphore, None
                );
            }
        }
        for &fence in &self.graphics_submit_fences {
            unsafe {
                self.vk.device().destroy_fence(fence, None);
            }
        }
        unsafe {
            self.vk.device().destroy_semaphore(self.sync_transfer_semaphore, None);
        }
        self.global_resources.write().unwrap().clean_up();
    }
}

#[derive(Default)]
struct RenderResult<'a> {
    frame_context: Option<FrameContext<'a>>,
    wait_semaphores: FixedVec<'a, vk::Semaphore, ArenaAlloc>,
    wait_values: FixedVec<'a, u64, ArenaAlloc>,
    wait_stages: FixedVec<'a, vk::PipelineStageFlags, ArenaAlloc>,
    signal_semaphores: FixedVec<'a, vk::Semaphore, ArenaAlloc>,
    signal_values: FixedVec<'a, u64, ArenaAlloc>,
    transfer_fence: vk::Fence,
}
