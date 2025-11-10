pub mod frame_graph;
pub mod pipeline;
pub mod image;
pub mod memory_binder;
pub mod linear_device_alloc;

mod memory_layout;
mod handle;
mod helpers;
mod shader_fn;
mod shader;
mod enums;
mod structs;
mod physical_device;
mod vulkan_context;
mod swapchain_context;
mod thread_context;
mod frame_state;
mod buffer;
mod global_resources;
mod commands;

mod swapchain_pass;

use std::{
    rc::Rc, sync::{Arc, RwLock},
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

use nox_mem::{
    string_types::*,
    vec_types::{ArrayVec, FixedVec, GlobalVec, Vector},
};

use nox_alloc::arena_alloc::*;

use super::{
    Interface,
    Version,
    AppName,
    error::Error,
    utility::clamp,
};

pub use vk::Format as VkFormat;

pub use enums::*;
pub use structs::*;
pub use memory_layout::MemoryLayout;
pub use handle::{Handle, RaiiHandle};
pub use image::*;
pub use buffer::*;
pub use physical_device::QueueFamilyIndices;
pub use global_resources::*;
pub use pipeline::*;
pub use commands::*;
pub use nox_derive::VertexInput;
pub use shader::*;
pub use pipeline::vertex_input::*;
pub use frame_graph::*;
use linear_device_alloc::LinearDeviceAlloc;

//use device_allocators::DeviceAllocators;
use vulkan_context::VulkanContext;
use swapchain_context::SwapchainContext;
use physical_device::PhysicalDeviceInfo;
use frame_state::FrameState;
use frame_graph::FrameGraphImpl;
use swapchain_context::PresentResult;
use thread_context::ThreadContext;

use swapchain_pass::SwapchainPassPipelineData;

runtime_token!(pub(crate) FrameToken);

pub type DeviceName = ArrayString<{ash::vk::MAX_PHYSICAL_DEVICE_NAME_SIZE}>;

pub const MIN_BUFFERED_FRAMES: u32 = 2;
pub const MAX_BUFFERED_FRAMES: u32 = 8;

pub struct Allocators {
    swapchain: ArenaAlloc,
    frame_graphs: UnsafeCell<ArrayVec<ArenaAlloc, {MAX_BUFFERED_FRAMES as usize}>>,
    _memory_layout: MemoryLayout,
}

impl Allocators {

    pub fn new(memory_layout: MemoryLayout) -> Option<Self> {
        Some(Self {
            swapchain: ArenaAlloc::new(memory_layout.swapchain_size())?,
            frame_graphs: UnsafeCell::new(Default::default()),
            _memory_layout: memory_layout,
        })
    }

    fn realloc_frame_graphs(&self, buffered_frames: u32) {
        assert!(buffered_frames <= MAX_BUFFERED_FRAMES);
        unsafe { &mut *self.frame_graphs.get() }.resize_with(
            buffered_frames as usize,
            || ArenaAlloc::new(self._memory_layout.frame_graph_arena_size()).unwrap()
        ).unwrap();
    }

    fn frame_graphs(&self) -> &ArrayVec<ArenaAlloc, {MAX_BUFFERED_FRAMES as usize}> {
        unsafe { &*self.frame_graphs.get() }
    }
}


pub struct RendererContext {
    global_resources: Arc<RwLock<GlobalResources>>,
    transfer_requests: Rc<UnsafeCell<TransferRequests>>,
    frame_buffer_size: image::Dimensions,
    pub(crate) device: Arc<ash::Device>,
    physical_device_info: Arc<PhysicalDeviceInfo>,
}

impl RendererContext {

    #[inline(always)]
    pub fn edit_resources(
        &self,
        mut f: impl FnMut(&mut GlobalResources) -> Result<(), Error>,
    ) -> Result<(), Error>
    {
        let mut resources = self.global_resources.write().unwrap();
        f(&mut resources)
    }

    #[inline(always)]
    pub fn edit_transfer_requests(
        &mut self,
        mut f: impl FnMut(&mut TransferRequests),
    )
    {
        let requests = unsafe {
            &mut *self.transfer_requests.get()
        };
        f(requests)
    }

    #[inline(always)]
    pub fn frame_buffer_size(&self) -> image::Dimensions {
        self.frame_buffer_size
    }

    #[inline(always)]
    pub fn buffer_size(&self, buffer: BufferId) -> Option<u64> {
        self.global_resources.read().unwrap().buffer_size(buffer)
    }

    #[inline(always)]
    pub fn create_linear_device_alloc_mappable(&self, block_size: u64) -> Result<LinearDeviceAlloc, Error> {
        LinearDeviceAlloc::new(
            self.device.clone(),
            block_size,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
            &self.physical_device_info,
            true
        )
    }
}

#[derive(Clone, Copy)]
pub struct ComputeState {
    command_buffer: vk::CommandBuffer,
    semaphore: vk::Semaphore,
    timeline_value: u64,
}

pub(crate) struct Renderer<'a> {
    transfer_commands: GlobalVec<TransferCommands>,
    transfer_requests: Rc<UnsafeCell<TransferRequests>>,
    sync_transfer_semaphore: vk::Semaphore,
    sync_transfer_timeline_value: u64,
    sync_transfer_commands: GlobalVec<TransferCommands>,
    main_thread_context: ThreadContext,
    frame_states: ArrayVec<Rc<TokenCell<FrameState, FrameToken>>, {MAX_BUFFERED_FRAMES as usize}>,
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

impl<'a> Renderer<'a> {

    pub fn new(
        window: &Window,
        app_name: &AppName,
        app_version: Version,
        enable_validation: bool,
        memory_layout: MemoryLayout,
        mut buffered_frames: u32,
        allocators: &'a Allocators,
    ) -> Result<Self, String>
    {
        buffered_frames = clamp(buffered_frames, MIN_BUFFERED_FRAMES, MAX_BUFFERED_FRAMES);
        assert!(buffered_frames <= MAX_BUFFERED_FRAMES);
        allocators.realloc_frame_graphs(buffered_frames);
        let tmp_alloc = Arc::new(ArenaAlloc::new(memory_layout.tmp_arena_size()).ok_or(format!("failed to create tmp alloc"))?);
        let vulkan_context = VulkanContext
            ::new(
                window,
                &app_name,
                app_version,
                buffered_frames,
                enable_validation,
                &tmp_alloc
            )
            .map_err(|e| e)?;
        let device = Arc::new(vulkan_context.device().clone());
        let main_thread_context = ThreadContext
            ::new(
                device.clone(),
                vulkan_context.queue_family_indices())
            .map_err(|e|
                format!("failed to create main thread context ( {:?} )", e)
            )?;
        let physical_device_info = vulkan_context.physical_device_info().clone();
        let global_resources = Arc::new(RwLock::new(
            GlobalResources
                ::new(
                    device.clone(),
                    Arc::new(vulkan_context.instance().clone()),
                    vulkan_context.physical_device(),
                    physical_device_info.clone(),
                    memory_layout
                )
                .map_err(|e| format!("failed to create global resources ( {:?} ) ", e))?
        ));
        let swapchain_pass_pipeline_data = SwapchainPassPipelineData
            ::new(global_resources.clone(), buffered_frames, &tmp_alloc)
            .map_err(|e| format!("failed to create full screen pass data ( {:?} )", e))?;
        let mut s = Self {
            main_thread_context,
            vulkan_context,
            frame_states: Default::default(),
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
            transfer_requests: Rc::new(UnsafeCell::new(TransferRequests::new())),
        };
        let mut frame_states = ArrayVec::new();
        let mut i = 0;
        frame_states.resize_with(
            buffered_frames as usize,
            || {
                let device_alloc = LinearDeviceAlloc::new(
                    device.clone(),
                    memory_layout.frame_graph_device_block_size(),
                    vk::MemoryPropertyFlags::DEVICE_LOCAL,
                    vk::MemoryPropertyFlags::LAZILY_ALLOCATED | vk::MemoryPropertyFlags::PROTECTED,
                    &physical_device_info,
                    false,
                ).unwrap();
                let s = Rc::new(TokenCell::new(
                    FrameState::new(device.clone(), global_resources.clone(), device_alloc).unwrap(),
                    &s.frame_token,
                ));
                i += 1;
                s
            }
        ).unwrap();
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
                return Err(format!("failed to allocate compute command buffers {:?}", result))
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
        s.frame_states = frame_states;
        Ok(s)
    }

    #[inline(always)]
    pub fn device_info(&self) -> &PhysicalDeviceInfo {
        self.vulkan_context.physical_device_info()
    }

    #[inline(always)]
    pub fn renderer_context(&mut self) -> RendererContext {
        RendererContext {
            global_resources: self.global_resources.clone(),
            transfer_requests: self.transfer_requests.clone(),
            frame_buffer_size: self.frame_buffer_size,
            device: self.device.clone(),
            physical_device_info: self.vulkan_context.physical_device_info_owned(),
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
    ) -> Result<(), Error>
    {
        let requests = unsafe {
            &mut * self.transfer_requests.get()
        };
        let count = requests.async_request_count();

        if count == 0 {
            return Ok(())
        }

        let device = self.device.clone();
        let transfer_commands = &mut self.transfer_commands;
        let queue_families = self.vulkan_context.queue_family_indices();
        let global_resources = self.global_resources.clone();

        let transfer_command_pool = Arc::new(TransientCommandPool::new(device.clone(), queue_families.transfer_index())?);
        let graphics_command_pool = Arc::new(TransientCommandPool::new(device.clone(), queue_families.graphics_index())?);

        let arena_guard = ArenaGuard::new(&self.tmp_alloc);

        let mut transfer_command_buffers = FixedVec::with_len(count as usize, Default::default(), &arena_guard)?;

        let mut alloc_info = vk::CommandBufferAllocateInfo {
            s_type: vk::StructureType::COMMAND_BUFFER_ALLOCATE_INFO,
            command_pool: transfer_command_pool.handle(),
            level: vk::CommandBufferLevel::PRIMARY,
            command_buffer_count: count,
            ..Default::default()
        };

        helpers::allocate_command_buffers(&device, &alloc_info, &mut transfer_command_buffers)?;

        let mut graphics_command_buffers = FixedVec::with_len(count as usize, Default::default(), &arena_guard)?;

        alloc_info.command_pool = graphics_command_pool.handle();

        helpers::allocate_command_buffers(&device, &alloc_info, &mut graphics_command_buffers)?;

        for (i, (id, staging_alloc)) in requests.async_transfer_iter().enumerate() {

            let alloc = global_resources
                .write()
                .unwrap()
                .lock_linear_device_alloc(staging_alloc)?;

            let mut commands = transfer_commands.push(TransferCommands::new(
                transfer_command_pool.clone(),
                transfer_command_buffers[i],
                graphics_command_pool.clone(),
                graphics_command_buffers[i],
                global_resources.clone(),
                alloc,
                id
            )?);

            interface
                .transfer_commands(id, &mut commands)?;
        }

        requests.clear();

        Ok(())
    }

    pub(crate) fn render<I: Interface>(
        &mut self,
        window: &Window,
        interface: &mut I,
        allocators: &'a Allocators,
    ) -> Result<(), String>
    {
        self
            .async_transfer_requests(interface)
            .map_err(|e| format!("failed to record async transfer requests: {:?}", e))?;
        let transfer_queue = self.vulkan_context.transfer_queue();
        let graphics_queue = self.vulkan_context.graphics_queue();
        let device = self.device.clone();
        let mut pending_transfers = Default::default();
        if !self.transfer_commands.is_empty() {
            let transfer_commands = &mut self.transfer_commands;
            pending_transfers = GlobalVec::with_capacity(transfer_commands.len());
            let mut ready_transfers = GlobalVec::with_capacity(transfer_commands.len());
            for (i, command) in transfer_commands.iter_mut().enumerate() {
                let (new, fence) = command
                    .get_transfer_fence()
                    .map_err(|e| format!("failed to create fence: {:?}", e))?;
                if new {
                    let command_buffer = command.transfer_command_buffer();
                    unsafe {
                        device
                            .end_command_buffer(command_buffer)
                            .map_err(|e| format!("failed to end command buffer: {:?}", e))?;
                    }
                    let submit_info = vk::SubmitInfo {
                        s_type: vk::StructureType::SUBMIT_INFO,
                        command_buffer_count: 1,
                        p_command_buffers: &command_buffer,
                        ..Default::default()
                    };
                    unsafe {
                        device.queue_submit(
                            transfer_queue,
                            &[submit_info],
                            fence,
                        ).map_err(|e| format!("failed to submit transfer commands: {:?}", e))?;
                    };
                }
                let (new, fence) = command
                    .get_graphics_fence()
                    .map_err(|e| format!("failed to create fence: {:?}", e))?;
                if new {
                    let command_buffer = command.graphics_command_buffer();
                    unsafe {
                        device.end_command_buffer(command_buffer)
                        .map_err(|e| format!("failed to end command buffer: {:?}", e))?;
                    }
                    let submit_info = vk::SubmitInfo {
                        s_type: vk::StructureType::SUBMIT_INFO,
                        command_buffer_count: 1,
                        p_command_buffers: &command_buffer,
                        ..Default::default()
                    };
                    unsafe {
                        device.queue_submit(
                            graphics_queue,
                            &[submit_info],
                            fence,
                        ).map_err(|e| format!("failed to submit graphics commands: {:?}", e))?;
                    }
                }
                unsafe {
                    let mut transfer_ready = false;
                    let mut graphics_ready = false;
                    match device.wait_for_fences(&[fence], true, 0) {
                        Ok(()) => {
                            transfer_ready = true;
                        },
                        Err(vk::Result::TIMEOUT) => {}
                        Err(e) => {
                            return Err(format!("unexpected fence wait error: {:?}", e))
                        }
                    }
                    match device.wait_for_fences(&[fence], true, 0) {
                        Ok(()) => {
                            graphics_ready = true;
                        },
                        Err(vk::Result::TIMEOUT) => {},
                        Err(e) => {
                            return Err(format!("unexpected fence wait error: {:?}", e))
                        }
                    }
                    if transfer_ready && graphics_ready {
                        ready_transfers.push(i);
                    } else {
                        pending_transfers.push(command.id());
                    }
                }
            }
            for i in ready_transfers.iter().rev() {
                transfer_commands.remove(*i);
            }
        }
        let swapchain_loader = self.vulkan_context.swapchain_loader().clone();
        let queue_family_indices = self.vulkan_context.queue_family_indices();
        let graphics_queue = self.vulkan_context.graphics_queue();
        let compute_queue = self.vulkan_context.compute_queue();
        let mut render_context = self.renderer_context();
        let (swapchain_context, recreated) = self.vulkan_context
            .get_swapchain_context(
                self.main_thread_context.graphics_pool(),
                &self.tmp_alloc,
                allocators
            )
            .map_err(|e| {
                format!("failed to get swapchain context ( {} )", e)
            })?;
        let frame_data = match swapchain_context
            .setup_image(&device, &swapchain_loader)
            .map_err(|e| {
                format!("failed to setup render image ( {} )", e)
            })? {
                Some(r) => r,
                None => return Ok(())
            };
        if recreated {
            self.frame_buffer_size = frame_data.extent.into();
            render_context.frame_buffer_size = self.frame_buffer_size;
            interface
                .frame_buffer_size_callback(&mut render_context)
                .map_err(|e| format!("interface frame buffer size callback failed ( {:?} )", e))?;
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
            .compute(&mut compute_commands)
            .map_err(|e| format!("interface failed to record compute commands ( {:?} )", e))?;
        unsafe {
            device.end_command_buffer(compute_state.command_buffer).unwrap();
        }
        let wait_value = compute_state.timeline_value;
        let signal_value = compute_state.timeline_value + 1;
        let mut timeline_submit = vk::TimelineSemaphoreSubmitInfo {
            s_type: vk::StructureType::TIMELINE_SEMAPHORE_SUBMIT_INFO,
            wait_semaphore_value_count: 1,
            p_wait_semaphore_values: &wait_value,
            signal_semaphore_value_count: 1,
            p_signal_semaphore_values: &signal_value,
            ..Default::default()
        };
        let wait_stage = vk::PipelineStageFlags::TOP_OF_PIPE;
        let compute_submit = vk::SubmitInfo {
            s_type: vk::StructureType::SUBMIT_INFO,
            command_buffer_count: 1,
            p_command_buffers: &compute_state.command_buffer,
            wait_semaphore_count: 1,
            p_wait_semaphores: &compute_state.semaphore,
            p_wait_dst_stage_mask: &wait_stage,
            signal_semaphore_count: 1,
            p_signal_semaphores: &compute_state.semaphore,
            ..Default::default()
        }.push_next(&mut timeline_submit);
        unsafe {
            let result = device.queue_submit(
                compute_queue,
                &[compute_submit],
                Default::default(),
            );
            if let Err(err) = result {
                return Err(format!("failed to submit to compute queue ( {:?} )", err))
            }
        }
        if let Err(e) = helpers::begin_command_buffer(&device, frame_data.command_buffer) {
            return Err(format!("failed to begin command buffer {:?}", e))
        }
        let pipeline = self.swapchain_pass_pipeline_data
            .get_pipeline(frame_data.format, &self.tmp_alloc)
            .map_err(|e| format!("failed to get full screen pass pipeline {:?}", e))?;
        let alloc = &allocators.frame_graphs()[frame_data.frame_index as usize];
        unsafe {
            alloc.force_clear();
        }
        self.sync_transfer_commands.clear();
        let (transfer_fence, mut wait_semaphores, mut wait_values, mut wait_stages, mut signal_semaphores, mut signal_values) =
        {
            let mut frame_graph = FrameGraphImpl::new(
                self.frame_states[frame_data.frame_index as usize].clone(),
                frame_data.extent.into(),
                frame_data.command_buffer,
                alloc,
                frame_data.frame_index,
                queue_family_indices,
                &mut self.frame_token,
            );
            interface
                .render(&mut frame_graph, &pending_transfers)
                .map_err(|e| format!("interface failed to render ( {:?} )", e))?;
            let mut render_commands = RenderCommands::new(
                device.clone(),
                frame_data.command_buffer,
                queue_family_indices,
                self.global_resources.clone(),
                compute_state.semaphore,
                compute_state.timeline_value,
                &self.tmp_alloc,
                frame_data.extent.into(),
                self.buffered_frames,
            ).map_err(|e| {
                format!("failed to create render commands: {:?}", e)
            })?;
            frame_graph
                .render(interface, &mut render_commands)
                .map_err(|e| format!("frame graph failed to render: {:?}", e))?;
            let (transfer_fence, wait_semaphores, wait_values, wait_stages) =
                if render_commands.transfer_commands.is_empty() {
                    (
                        vk::Fence::null(),
                        FixedVec
                            ::with_capacity(2, alloc)
                            .map_err(|e| format!("failed to allocate wait semaphores: {:?}", e))?,
                        FixedVec
                            ::with_capacity(2, alloc)
                            .map_err(|e| format!("failed to allocate wait semaphore values: {:?}", e))?,
                        FixedVec
                            ::with_capacity(2, alloc)
                            .map_err(|e| format!("failed to allocate wait semaphore stages: {:?}", e))?,
                    )
                } else {
                    let mut transfer_command_buffers = FixedVec::with_capacity(render_commands.transfer_commands.len(), alloc)
                        .map_err(|e| format!("failed to allocate command buffers: {:?}", e))?;
                    let mut graphics_command_buffers = FixedVec::with_capacity(render_commands.transfer_commands.len(), alloc)
                        .map_err(|e| format!("failed to allocate command buffers: {:?}", e))?;
                    for transfer_commands in &render_commands.transfer_commands {
                        let command_buffer = transfer_commands.transfer_command_buffer();
                        unsafe {
                            device.end_command_buffer(command_buffer)
                                .map_err(|e| format!("failed to end transfer command buffer: {:?}", e))?;
                        }
                        transfer_command_buffers.push(command_buffer).ok();
                        let command_buffer = transfer_commands.graphics_command_buffer();
                        unsafe {
                            device.end_command_buffer(command_buffer)
                                .map_err(|e| format!("failed to end graphics command buffer: {:?}", e))?;
                        }
                        graphics_command_buffers.push(command_buffer).ok();
                    }
                    let fence_info = vk::FenceCreateInfo {
                        s_type: vk::StructureType::FENCE_CREATE_INFO,
                        ..Default::default()
                    };
                    let transfer_fence = unsafe {
                        device.create_fence(&fence_info, None)
                    }.map_err(|e| format!("unexpected error when creating fence: {:?}", e))?;
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
                        ).map_err(|e| format!("failed to submit transfer commands: {:?}", e))?;
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
                        ).map_err(|e| format!("failed t o submit graphics commands: {:?}", e))?;
                    }
                    let mut wait_semaphores = FixedVec
                        ::with_capacity(3, alloc)
                        .map_err(|e| format!("failed to allocate wait semaphores: {:?}", e))?;
                    wait_semaphores.push(self.sync_transfer_semaphore).ok();
                    let mut wait_values = FixedVec
                        ::with_capacity(3, alloc)
                        .map_err(|e| format!("failed to allocate wait semaphore values: {:?}", e))?;
                    wait_values.push(self.sync_transfer_timeline_value).ok();
                    let mut wait_stages = FixedVec
                        ::with_capacity(3, alloc)
                        .map_err(|e| format!("failed to allocate wait semaphore stages: {:?}", e))?;
                    wait_stages.push(vk::PipelineStageFlags::TRANSFER).ok();
                    (transfer_fence, wait_semaphores, wait_values, wait_stages)
                };
            let signal_count = frame_graph.signal_semaphore_count() as usize;
            let mut signal_semaphores = FixedVec
                ::with_capacity(signal_count + 2, alloc)
                .map_err(|e| format!("failed to allocate signal semaphores: {:?}", e))?;
            let mut signal_values = FixedVec
                ::with_capacity(signal_count + 2, alloc)
                .map_err(|e| format!("failed to allocate signal semaphore values: {:?}", e))?;
            let g = self.global_resources.read().unwrap();
            frame_graph.collect_signal_semaphores(|id, value| {
                let handle = g.get_timeline_semaphore(id)?;
                signal_semaphores.push(handle)?;
                signal_values.push(value)?;
                Ok(())
            }).map_err(|e| format!("failed to collect signal semaphores ( {:?} )", e))?;
            self.sync_transfer_commands.move_from_vec(&mut render_commands.transfer_commands);
            (transfer_fence, wait_semaphores, wait_values, wait_stages, signal_semaphores, signal_values)
        };
        let frame_state = self.frame_states[frame_data.frame_index as usize].borrow_mut(&mut self.frame_token);
        let mut image_state = frame_data.image_state;
        let graphics_queue_index = queue_family_indices.graphics_index();
        if let Some((render_image, range_info)) = frame_state
                .get_render_image(queue_family_indices.graphics_index())
                .map_err(|e| format!("failed to get render image ( {:?} )", e))?
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
                frame_state.render_done();
            }
        }
        let (semaphores, fence) = swapchain_context
            .setup_submit(
                &device,
                image_state,
                queue_family_indices.graphics_index()
            );
        if let Err(e) = unsafe { device.end_command_buffer(frame_data.command_buffer) } {
            return Err(format!("failed to end command buffer {:?}", e))
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
        if let Err(e) = unsafe { device.queue_submit(graphics_queue, &[submit_info], fence) } {
            return Err(format!("graphics queue submit failed {:?}", e))
        }
        if !self.sync_transfer_commands.is_empty() {
            unsafe {
                device
                    .wait_for_fences(&[transfer_fence], true, u64::MAX)
                    .map_err(|e| format!("failed to wait for transfer fence: {:?}", e))?;
                device.destroy_fence(transfer_fence, None);
            }
        }
        let present_result = swapchain_context
            .present_submit(&swapchain_loader, graphics_queue)
            .map_err(|e| format!("queue present failed {}", e))?;
        if present_result != PresentResult::Success || frame_data.suboptimal {
            self.vulkan_context.request_swapchain_update(self.buffered_frames, window.inner_size());
        }
        self.compute_states[frame_data.frame_index as usize].timeline_value += 2;
        Ok(())
    }

    pub(crate) fn clean_up(&mut self, allocators: &'a Allocators) {
        println!("Nox renderer message: terminating renderer");
        unsafe {
            self.device.device_wait_idle().ok();
        }
        for state in &self.frame_states {
            unsafe {
                state.borrow_mut(&mut self.frame_token).force_clean_up();
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
        self.vulkan_context.destroy_swapchain(self.main_thread_context.graphics_pool(), &allocators);
    }
}
