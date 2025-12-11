pub mod frame_graph;
mod pipeline;
mod image;
pub mod memory_binder;
pub mod linear_device_alloc;
mod context;

mod memory_layout;
mod handle;
mod helpers;
mod shader;
mod enums;
mod structs;
mod physical_device;
mod vulkan_context;
mod swapchain_context;
mod thread_context;
mod frame_context;
mod buffer;
mod global_resources;
mod commands;

mod swapchain_pass;

use std::{
    sync::{Arc, RwLock},
};

use core::{
    cell::{UnsafeCell},
};

use ash::vk;

use winit::{
    dpi::PhysicalSize, window::Window
};

use token_cell::{
    prelude::*,
    runtime_token,
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
    error::{self, Result, Error, Context, ErrorContext, location},
    format_location,
};

pub use vk::Format as VkFormat;

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
pub use nox_derive::VertexInput;
pub use shader::*;
pub use pipeline::vertex_input::*;
pub use frame_graph::*;
use linear_device_alloc::LinearDeviceAlloc;

use vulkan_context::VulkanContext;
use swapchain_context::SwapchainContext;
use frame_context::{FrameContext, ResourcePool};
use swapchain_context::PresentResult;
use thread_context::ThreadContext;

use swapchain_pass::SwapchainPassPipelineData;

runtime_token!(pub(crate) FrameToken);

pub type DeviceName = ArrayString<{ash::vk::MAX_PHYSICAL_DEVICE_NAME_SIZE}>;

pub const MIN_BUFFERED_FRAMES: u32 = 2;
pub const MAX_BUFFERED_FRAMES: u32 = 8;

pub struct HostAllocators {
    swapchain: ArenaAlloc,
    frame_graphs: UnsafeCell<ArrayVec<ArenaAlloc, {MAX_BUFFERED_FRAMES as usize}>>,
    _memory_layout: MemoryLayout,
}

impl HostAllocators {

    pub fn new(memory_layout: MemoryLayout) -> Result<Self> {
        Ok(Self {
            swapchain: ArenaAlloc
                ::new(memory_layout.swapchain_size())
                .context_with(|| format_compact!("failed to create arena alloc at {}", location!()))?,
            frame_graphs: UnsafeCell::new(Default::default()),
            _memory_layout: memory_layout,
        })
    }

    fn realloc_frame_graphs(&self, buffered_frames: u32) -> Result<()> {
        assert!(buffered_frames <= MAX_BUFFERED_FRAMES);
        unsafe { &mut *self.frame_graphs.get() }.try_resize_with(
            buffered_frames as usize,
            || ArenaAlloc
                ::new(self._memory_layout.frame_graph_arena_size())
                .context_with(|| format_compact!("failed to create arena alloc at {}", location!())),
            |err| Error::new(ErrorContext::VecError(location!()), err)
        )
    }

    fn frame_graphs(&self) -> &ArrayVec<ArenaAlloc, {MAX_BUFFERED_FRAMES as usize}> {
        unsafe { &*self.frame_graphs.get() }
    }
}

#[derive(Clone, Copy)]
pub struct ComputeState {
    command_buffer: vk::CommandBuffer,
    semaphore: vk::Semaphore,
    timeline_value: u64,
}

pub(crate) struct Gpu<'a> {
    transfer_commands: GlobalVec<TransferCommandsStorage>,
    transfer_requests: TransferRequests,
    sync_transfer_semaphore: vk::Semaphore,
    sync_transfer_timeline_value: u64,
    sync_transfer_commands: GlobalVec<TransferCommandsStorage>,
    main_thread_context: ThreadContext,
    frame_resource_pools: ArrayVec<ResourcePool, {MAX_BUFFERED_FRAMES as usize}>,
    compute_states: ArrayVec<ComputeState, {MAX_BUFFERED_FRAMES as usize}>,
    swapchain_pass_pipeline_data: SwapchainPassPipelineData,
    global_resources: Arc<RwLock<GlobalResources>>,
    device: Arc<ash::Device>,
    vulkan_context: VulkanContext<'a>,
    _memory_layout: MemoryLayout,
    buffered_frames: u32,
    tmp_alloc: Arc<ArenaAlloc>,
    frame_buffer_size: image::Dimensions,
    frame_token: FrameToken,
}

impl<'a> Gpu<'a> {

    pub fn new(
        window: &Window,
        app_name: &AppName,
        app_version: Version,
        enable_validation: bool,
        memory_layout: MemoryLayout,
        mut buffered_frames: u32,
        host_allocators: &'a HostAllocators,
    ) -> Result<Self>
    {
        buffered_frames = clamp(buffered_frames, MIN_BUFFERED_FRAMES, MAX_BUFFERED_FRAMES);
        assert!(buffered_frames <= MAX_BUFFERED_FRAMES);
        host_allocators.realloc_frame_graphs(buffered_frames);
        let tmp_alloc = Arc::new(
            ArenaAlloc
                ::new(memory_layout.tmp_arena_size())
                .context_with(|| format_compact!("failed to create arena alloc at {}", location!()))?
        );
        let vulkan_context = VulkanContext
            ::new(
                window,
                &app_name,
                app_version,
                buffered_frames,
                enable_validation,
                &tmp_alloc
            ).context("failed to create vulkan backend")?;
        let device = Arc::new(vulkan_context.device().clone());
        let main_thread_context = ThreadContext
            ::new(
                device.clone(),
                vulkan_context.queue_family_indices())
            .context("failed to create main thread context")?;
        let physical_device_info = vulkan_context.physical_device_info().clone();
        let global_resources = Arc::new(RwLock::new(
            GlobalResources
                ::new(
                    device.clone(),
                    Arc::new(vulkan_context.instance().clone()),
                    vulkan_context.physical_device(),
                    physical_device_info.clone(),
                    memory_layout
                ).context("failed to initialize global resources")?
        ));
        let swapchain_pass_pipeline_data = SwapchainPassPipelineData
            ::new(global_resources.clone(), buffered_frames, &tmp_alloc)
            .context("failed to create main render pass")?;
        let mut s = Self {
            main_thread_context,
            vulkan_context,
            frame_resource_pools: Default::default(),
            compute_states: Default::default(),
            swapchain_pass_pipeline_data,
            global_resources: global_resources.clone(),
            device: device.clone(),
            _memory_layout: memory_layout,
            buffered_frames,
            transfer_commands: Default::default(),
            sync_transfer_semaphore: Default::default(),
            sync_transfer_timeline_value: 0,
            sync_transfer_commands: Default::default(),
            tmp_alloc,
            frame_buffer_size: image::Dimensions::new(1, 1, 1),
            frame_token: FrameToken::new().unwrap(),
            transfer_requests: Default::default(),
        };
        let mut frame_resource_pools = ArrayVec::new();
        let mut i = 0;
        frame_resource_pools.try_resize_with(
            buffered_frames as usize,
            || {
                let device_alloc = LinearDeviceAlloc::new(
                    device.clone(),
                    memory_layout.frame_graph_device_block_size(),
                    vk::MemoryPropertyFlags::DEVICE_LOCAL,
                    vk::MemoryPropertyFlags::LAZILY_ALLOCATED | vk::MemoryPropertyFlags::PROTECTED,
                    &physical_device_info,
                    false,
                ).context_with(|| format_compact!(
                    "failed to create linear device alloc at {}", location!(),
                ))?;
                let s = ResourcePool::new(device_alloc);
                i += 1;
                Ok(s)
            },
            |err| Error::new(ErrorContext::VecError(location!()),  err),
        )?;
        let mut compute_command_buffers = ArrayVec::<vk::CommandBuffer, {MAX_BUFFERED_FRAMES as usize}>::new();
        let mut compute_semaphores = ArrayVec::<vk::Semaphore, {MAX_BUFFERED_FRAMES as usize}>::new(); 
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
            let result = (device.fp_v1_0().allocate_command_buffers)(
                device.handle(),
                &alloc_info,
                compute_command_buffers.as_mut_ptr(),
            );
            if result != vk::Result::SUCCESS {
                return Err(Error::new("failed to allocate compute command buffers", result))
            }
            compute_command_buffers.set_len(buffered_frames as usize);
            for _ in 0..buffered_frames {
                let fence = device.create_semaphore(&semaphore_info, None).unwrap();
                compute_semaphores.push(fence).unwrap();
            }
            s.sync_transfer_semaphore = device.create_semaphore(&semaphore_info, None).unwrap();
        }
        for (i, buffer) in compute_command_buffers.iter().enumerate() {
            s.compute_states.push(ComputeState {
                command_buffer: *buffer,
                semaphore: compute_semaphores[i],
                timeline_value: 0,
            }).unwrap();
        }
        s.frame_resource_pools = frame_resource_pools;
        Ok(s)
    }

    #[inline(always)]
    pub fn device_info(&self) -> &PhysicalDeviceInfo {
        self.vulkan_context.physical_device_info()
    }

    #[inline(always)]
    pub fn context(&mut self) -> GpuContext {
        GpuContext {
            global_resources: self.global_resources.write().unwrap(),
            transfer_requests: &mut self.transfer_requests,
            frame_buffer_size: self.frame_buffer_size,
        }
    }

    #[inline(always)]
    pub fn wait_idle(&self) {
        unsafe {
            self.device.device_wait_idle().ok();
        }
    }

    #[inline(always)]
    pub(crate) fn request_resize(&mut self, size: PhysicalSize<u32>) {
        self.vulkan_context.request_swapchain_update(self.buffered_frames, size);
    }

    fn async_transfer_requests<I: Interface>(
        &mut self,
        interface: &mut I,
    ) -> error::Result<()>
    {
        let count = self.transfer_requests.async_request_count();

        if count == 0 {
            return Ok(())
        }

        let device = self.device.clone();
        let queue_families = self.vulkan_context.queue_family_indices();
        let global_resources = self.global_resources.clone();

        let transfer_command_pool = Arc::new(TransientCommandPool
            ::new(device.clone(), queue_families.transfer_index())
            .context("failed to create transient transfer command pool")?
        );
        let graphics_command_pool = Arc::new(TransientCommandPool
            ::new(device.clone(), queue_families.graphics_index())
            .context("failed to create transient graphics command pool")?
        );

        let arena_guard = ArenaGuard::new(&self.tmp_alloc);

        let mut transfer_command_buffers = FixedVec
            ::with_len(count as usize, Default::default(), &arena_guard)
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
            ::with_len(count as usize, Default::default(), &arena_guard)
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

            let mut context = GpuContext::new(
                global_resources.write().unwrap(),
                &mut new_requests,
                self.frame_buffer_size,
            );

            let mut commands = TransferCommands::new(&mut storage, &mut context);

            interface
                .event(Event::TransferWork {
                    request_id: id,
                    commands: &mut commands,
                })
                .context_from_origin(|orig| ErrorContext::EventError(orig))?;

            self.transfer_commands.push(storage);
        }

        self.transfer_requests.clear();

        if !new_requests.is_empty() {
            self.transfer_requests = new_requests;
        }

        Ok(())
    }

    fn process_transfer_requests(
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
            let mut context = GpuContext::new(
                self.global_resources.write().unwrap(),
                &mut dummy_requests,
                Default::default(),
            );
            for i in 0..self.transfer_commands.len()
            {
                let mut commands = TransferCommands::new(&mut self.transfer_commands[i], &mut context);
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
                        commands.gpu().device()
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
                        commands.gpu().device().queue_submit(
                            transfer_queue,
                            &[submit_info],
                            sync_objects.transfer_fence,
                        ).context(ErrorContext::TransferQueueSubmitError(location!()))?;
                    };
                    unsafe {
                        commands.gpu().device()
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
                        commands.gpu().device().queue_submit(
                            graphics_queue,
                            &[submit_info],
                            sync_objects.graphics_fence,
                        ).context(ErrorContext::GraphicsQueueSubmitError(location!()))?;
                    }
                }
                unsafe {
                    let mut ready = false;
                    match commands.gpu().device().wait_for_fences(&[sync_objects.transfer_fence, sync_objects.graphics_fence], true, 0) {
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

    pub(crate) fn render(
        &mut self,
        window: &Window,
        interface: &mut impl Interface,
        host_allocators: &'a HostAllocators,
    ) -> error::Result<()>
    {
        let graphics_queue = self.vulkan_context.graphics_queue();
        let transfer_queue = self.vulkan_context.transfer_queue();
        let compute_queue = self.vulkan_context.compute_queue();
        self.async_transfer_requests(interface)
            .context("async transfer requests failed")?;
        let mut pending_transfers = GlobalVec::new();
        self.process_transfer_requests(transfer_queue, graphics_queue, &mut pending_transfers)
            .context("failed to process transfer requests")?;
        let device = self.device.clone();
        self.global_resources
            .write()
            .unwrap()
            .update_semaphores()
            .context("failed to update semaphores")?;
        let swapchain_loader = self.vulkan_context.swapchain_loader().clone();
        let queue_family_indices = self.vulkan_context.queue_family_indices();
        let (swapchain_context, recreated) = self.vulkan_context
            .get_swapchain_context(
                self.main_thread_context.graphics_pool(),
                &self.tmp_alloc,
                host_allocators,
            )?;
        let mut swapchain_context = swapchain_context.borrow_mut();
        let frame_data = match swapchain_context.setup_image(&device, &swapchain_loader)?
        {
            Some(r) => r,
            None => return Ok(())
        };
        if recreated {
            self.frame_buffer_size = frame_data.extent.into();
            let mut context = self.context();
            interface.event(Event::FrameBufferResized {
                gpu: &mut context,
                new_size: self.frame_buffer_size,
            }).context_from_origin(|orig| ErrorContext::EventError(orig))?;
            self.async_transfer_requests(interface)
                .context("async transfer requests failed")?;
            self.process_transfer_requests(transfer_queue, graphics_queue, &mut pending_transfers)?;
        } 
        let compute_state = self.compute_states[frame_data.frame_index as usize];
        unsafe {
            device.reset_command_buffer(
                compute_state.command_buffer, vk::CommandBufferResetFlags::RELEASE_RESOURCES
            ).unwrap();
        }
        helpers::begin_command_buffer(&device, compute_state.command_buffer).unwrap();
        let mut compute_commands = ComputeCommands::new(
            self.device.clone(),
            compute_state.command_buffer,
            self.global_resources.clone(),
            &self.tmp_alloc,
            queue_family_indices.compute_index(),
        );
        interface
            .event(Event::ComputeWork {
                commands: &mut compute_commands
            }).context_from_origin(|orig| ErrorContext::EventError(orig))?;
        unsafe {
            device.end_command_buffer(compute_state.command_buffer).unwrap();
        }
        {
            let tmp_alloc = ArenaGuard::new(&self.tmp_alloc);
            let wait_count = 1 + compute_commands.wait_semaphores.len();
            let signal_count = 1 + compute_commands.signal_semaphores.len();
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
                device.queue_submit(
                    compute_queue,
                    &[compute_submit],
                    Default::default(),
                ).context(ErrorContext::ComputeQueueSubmitError(location!()))?;
            }
        }
        helpers
            ::begin_command_buffer(&device, frame_data.command_buffer)
            .context_with(|| format_compact!("failed to begin command buffer at {}", location!()))?;
        let pipeline = self.swapchain_pass_pipeline_data
            .get_pipeline(frame_data.format, &self.tmp_alloc)?;
        let alloc = &host_allocators.frame_graphs()[frame_data.frame_index as usize];
        unsafe {
            alloc.force_clear();
        }
        self.sync_transfer_commands.clear();
        let (
            transfer_fence,
            mut wait_semaphores,
            mut wait_values, mut wait_stages,
            mut signal_semaphores,
            mut signal_values,
            mut frame_context,
        ) =
        {
            let mut frame_graph = FrameGraph::new(
                FrameContext::new(
                    frame_data.command_buffer,
                    device.clone(),
                    self.context(),
                    &mut self.frame_resource_pools[frame_data.frame_index as usize],
                ),
                frame_data.command_buffer,
                alloc,
                frame_data.frame_index,
                queue_family_indices,
            );
            interface
                .event(Event::Render {
                    frame_graph: &mut frame_graph,
                    pending_transfers: &pending_transfers,
                }).context_from_origin(|orig| ErrorContext::EventError(orig))?;
            let mut render_commands = RenderCommands::new(
                device.clone(),
                frame_data.command_buffer,
                queue_family_indices,
                frame_graph.gpu(),
                compute_state.semaphore,
                compute_state.timeline_value,
                &self.tmp_alloc,
                self.buffered_frames,
            )?;
            frame_graph.render(interface, &mut render_commands)?;
            let (transfer_fence, mut wait_semaphores, mut wait_values, mut wait_stages) =
                if render_commands.transfer_commands.is_empty() {
                    let frame_graph_count = frame_graph.wait_semaphore_count() as usize;
                    (
                        vk::Fence::null(),
                        FixedVec
                            ::with_capacity(frame_graph_count + 2, alloc)
                            .context_with(|| ErrorContext::VecError(location!()))?,
                        FixedVec
                            ::with_capacity(frame_graph_count + 2, alloc)
                            .context_with(|| ErrorContext::VecError(location!()))?,
                        FixedVec
                            ::with_capacity(frame_graph_count + 2, alloc)
                            .context_with(|| ErrorContext::VecError(location!()))?,
                    )
                } else {
                    let frame_graph_count = frame_graph.wait_semaphore_count() as usize;
                    let mut transfer_command_buffers = FixedVec
                        ::with_capacity(render_commands.transfer_commands.len(), alloc)
                        .context_with(|| ErrorContext::VecError(location!()))?;
                    let mut graphics_command_buffers = FixedVec
                        ::with_capacity(render_commands.transfer_commands.len(), alloc)
                        .context_with(|| ErrorContext::VecError(location!()))?;
                    for storage in &render_commands.transfer_commands {
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
                    self.sync_transfer_timeline_value += 1;
                    let mut timeline_info = vk::TimelineSemaphoreSubmitInfo {
                        s_type: vk::StructureType::TIMELINE_SEMAPHORE_SUBMIT_INFO,
                        signal_semaphore_value_count: 1,
                        p_signal_semaphore_values: &self.sync_transfer_timeline_value,
                        ..Default::default()
                    };
                    let submit_info = vk::SubmitInfo {
                        s_type: vk::StructureType::SUBMIT_INFO,
                        command_buffer_count: transfer_command_buffers.len() as u32,
                        p_command_buffers: transfer_command_buffers.as_ptr(),
                        signal_semaphore_count: 1,
                        p_signal_semaphores: &self.sync_transfer_semaphore,
                        ..Default::default()
                    }.push_next(&mut timeline_info);
                    unsafe {
                        device.queue_submit(
                            transfer_queue,
                            &[submit_info],
                            vk::Fence::null(),
                        ).context(ErrorContext::TransferQueueSubmitError(location!()))?;
                    };
                    let wait_value = self.sync_transfer_timeline_value;
                    let wait_stage = vk::PipelineStageFlags::TRANSFER;
                    self.sync_transfer_timeline_value += 1;
                    let mut timeline_info = vk::TimelineSemaphoreSubmitInfo {
                        s_type: vk::StructureType::TIMELINE_SEMAPHORE_SUBMIT_INFO,
                        wait_semaphore_value_count: 1,
                        p_wait_semaphore_values: &wait_value,
                        signal_semaphore_value_count: 1,
                        p_signal_semaphore_values: &self.sync_transfer_timeline_value,
                        ..Default::default()
                    };
                    let submit_info = vk::SubmitInfo {
                        s_type: vk::StructureType::SUBMIT_INFO,
                        command_buffer_count: graphics_command_buffers.len() as u32,
                        p_command_buffers: graphics_command_buffers.as_ptr(),
                        wait_semaphore_count: 1,
                        p_wait_semaphores: &self.sync_transfer_semaphore,
                        p_wait_dst_stage_mask: &wait_stage,
                        signal_semaphore_count: 1,
                        p_signal_semaphores: &self.sync_transfer_semaphore,
                        ..Default::default()
                    }.push_next(&mut timeline_info);
                    unsafe {
                        device.queue_submit(
                            graphics_queue,
                            &[submit_info],
                            transfer_fence,
                        ).context(ErrorContext::GraphicsQueueSubmitError(location!()))?;
                    }
                    let mut wait_semaphores = FixedVec
                        ::with_capacity(frame_graph_count + 3, alloc)
                        .context_with(|| ErrorContext::VecError(location!()))?;
                    wait_semaphores.push(self.sync_transfer_semaphore).ok();
                    let mut wait_values = FixedVec
                        ::with_capacity(frame_graph_count + 3, alloc)
                        .context_with(|| ErrorContext::VecError(location!()))?;
                    wait_values.push(self.sync_transfer_timeline_value).ok();
                    let mut wait_stages = FixedVec
                        ::with_capacity(frame_graph_count + 3, alloc)
                        .context_with(|| ErrorContext::VecError(location!()))?;
                    wait_stages.push(vk::PipelineStageFlags::TRANSFER).ok();
                    (transfer_fence, wait_semaphores, wait_values, wait_stages)
                };
            let signal_count = frame_graph.signal_semaphore_count() as usize;
            let mut signal_semaphores = FixedVec
                ::with_capacity(signal_count + 2, alloc)
                .context_with(|| ErrorContext::VecError(location!()))?;
            let mut signal_values = FixedVec
                ::with_capacity(signal_count + 2, alloc)
                .context_with(|| ErrorContext::VecError(location!()))?;
            let g = self.global_resources.read().unwrap();
            frame_graph.collect_semaphores(
                |id, value| {
                    let handle = g.get_timeline_semaphore(id)?;
                    signal_semaphores.push(handle).unwrap();
                    signal_values.push(value).unwrap();
                    Ok(())
                },
                |id, value, stage| {
                    let handle = g.get_timeline_semaphore(id)?;
                    wait_semaphores.push(handle).unwrap();
                    wait_values.push(value).unwrap();
                    wait_stages.push(stage.into()).unwrap();
                    Ok(())
                }
            )?;
            self.sync_transfer_commands.move_from_vec(&mut render_commands.transfer_commands);
            (
                transfer_fence,
                wait_semaphores, wait_values, wait_stages,
                signal_semaphores, signal_values,
                frame_graph.finalize(),
            )
        };
        let mut image_state = frame_data.image_state;
        let graphics_queue_index = queue_family_indices.graphics_index();
        if let Some((render_image, range_info)) = frame_context
            .get_render_image(queue_family_indices.graphics_index())?
        {
            let command_buffer = frame_data.command_buffer;
            let write_image_state = ImageState::new(
                vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
                vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                graphics_queue_index,
                vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            );
            let memory_barrier = image_state.to_memory_barrier(
                frame_data.image,
                write_image_state,
                SwapchainContext::subresource_range_info(),
            );
            unsafe {
                device.cmd_pipeline_barrier(
                    command_buffer,
                    image_state.pipeline_stage,
                    write_image_state.pipeline_stage,
                    Default::default(), Default::default(), Default::default(),
                    &[memory_barrier]
                );
            };
            image_state = write_image_state;
            let render_area = vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: frame_data.extent,
            };
            let color_attachment = vk::RenderingAttachmentInfo {
                s_type: vk::StructureType::RENDERING_ATTACHMENT_INFO,
                image_view: frame_data.image_view,
                image_layout: image_state.layout,
                load_op: vk::AttachmentLoadOp::CLEAR,
                store_op: vk::AttachmentStoreOp::STORE,
                ..Default::default()
            };
            let rendering_info = vk::RenderingInfo {
                s_type: vk::StructureType::RENDERING_INFO,
                render_area,
                layer_count: 1,
                color_attachment_count: 1,
                p_color_attachments: &color_attachment,
                ..Default::default()
            };
            unsafe {

                device.cmd_set_viewport(
                    command_buffer, 0,
                    &[
                        vk::Viewport {
                            x: 0.0,
                            y: 0.0,
                            width: frame_data.extent.width as f32,
                            height: frame_data.extent.height as f32,
                            min_depth: 0.0,
                            max_depth: 1.0,
                        }
                    ]
                );
                device.cmd_set_scissor(
                    command_buffer, 0,
                    &[
                        vk::Rect2D {
                            offset: Default::default(),
                            extent: frame_data.extent,
                        },
                    ]
                );
                device.cmd_begin_rendering(command_buffer, &rendering_info);

                device.cmd_bind_pipeline(command_buffer, vk::PipelineBindPoint::GRAPHICS, pipeline);
                device.cmd_bind_descriptor_sets(
                    command_buffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    self.swapchain_pass_pipeline_data.get_pipeline_layout().unwrap(),
                    0,
                    &[self.swapchain_pass_pipeline_data.get_descriptor_set(
                        render_image,
                        range_info,
                        frame_data.frame_index,
                        &self.tmp_alloc
                    ).unwrap()],
                    Default::default(),
                );
                device.cmd_draw(command_buffer, 6, 1, 0, 0);
                device.cmd_end_rendering(command_buffer);
                frame_context.render_done();
            }
        }
        let (semaphores, fence) = swapchain_context
            .setup_submit(
                &device,
                image_state,
                queue_family_indices.graphics_index()
            );
        unsafe { device
            .end_command_buffer(frame_data.command_buffer)
            .context(ErrorContext::CommandBufferEndError(location!()))?;
        }
        wait_semaphores.append(&[semaphores.wait_semaphore, compute_state.semaphore]).unwrap();
        wait_values.append(&[0, compute_state.timeline_value + 1]).unwrap();
        wait_stages.append(&[semaphores.wait_stage, vk::PipelineStageFlags::COMPUTE_SHADER]).unwrap();
        signal_semaphores.append(&[semaphores.signal_semaphore, compute_state.semaphore]).unwrap();
        signal_values.append(&[0, compute_state.timeline_value + 2]).unwrap();
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
            p_command_buffers: &frame_data.command_buffer,
            ..Default::default()
        };
        submit_info = submit_info.push_next(&mut timeline_submit);
        if let Err(err) = unsafe { device.queue_submit(graphics_queue, &[submit_info], fence) } {
            return Err(Error::new(ErrorContext::GraphicsQueueSubmitError(location!()), err))
        }
        if !self.sync_transfer_commands.is_empty() {
            unsafe {
                device
                    .wait_for_fences(&[transfer_fence], true, u64::MAX)
                    .context_with(|| format_compact!("failed to wait for fences at {}", location!()))?;
                device.destroy_fence(transfer_fence, None);
            }
        }
        let present_result = swapchain_context.present_submit(&swapchain_loader, graphics_queue)?;
        if present_result != PresentResult::Success || frame_data.suboptimal {
            self.vulkan_context.request_swapchain_update(self.buffered_frames, window.inner_size());
        }
        self.compute_states[frame_data.frame_index as usize].timeline_value += 2;
        Ok(())
    }

    pub(crate) fn clean_up(&mut self, host_allocators: &'a HostAllocators) {
        log::info!("terminating renderer");
        unsafe {
            self.device.device_wait_idle().ok();
        }
        for pool in &mut self.frame_resource_pools {
            unsafe {
                pool.force_clean_up();
            }
        }
        for state in &self.compute_states {
            unsafe {
                self.device.destroy_semaphore(
                    state.semaphore, None
                );
            }
        }
        unsafe {
            self.device.destroy_semaphore(self.sync_transfer_semaphore, None);
        }
        self.global_resources.write().unwrap().clean_up();
        self.vulkan_context
            .destroy_swapchain(self.main_thread_context.graphics_pool(), &host_allocators);
    }
}
