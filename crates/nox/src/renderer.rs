pub mod frame_graph;
pub mod pipeline;
pub mod image;
pub mod memory_binder;

mod memory_layout;
mod handle;
mod helpers;
mod shader_fn;
mod shader;
mod enums;
mod physical_device;
mod vulkan_context;
mod swapchain_context;
mod thread_context;
mod frame_state;
mod linear_device_alloc;
mod buffer;
mod global_resources;
mod commands;

mod swapchain_pass;

use std::{
    rc::Rc, sync::{Arc, RwLock}, thread::{JoinHandle}
};

use core::{
    cell::{UnsafeCell, RefCell},
};

use ash::vk;

use nox_mem::{
    string_types::*,
    vec_types::{ArrayVec, GlobalVec, Vector},
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

//use device_allocators::DeviceAllocators;
use linear_device_alloc::LinearDeviceAlloc;
use vulkan_context::VulkanContext;
use swapchain_context::SwapchainContext;
use physical_device::PhysicalDeviceInfo;
use frame_state::FrameState;
use frame_graph::FrameGraphImpl;
use swapchain_context::PresentResult;
use thread_context::ThreadContext;

use swapchain_pass::SwapchainPassPipelineData;

use winit::{
    dpi::PhysicalSize, window::Window
};

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

    fn realloc_frame_graphs(&self, buffered_frame_count: u32) {
        assert!(buffered_frame_count <= MAX_BUFFERED_FRAMES);
        unsafe { &mut *self.frame_graphs.get() }.resize_with(
            buffered_frame_count as usize,
            || ArenaAlloc::new(self._memory_layout.frame_graph_arena_size()).unwrap()
        ).unwrap();
    }

    fn frame_graphs(&self) -> &ArrayVec<ArenaAlloc, {MAX_BUFFERED_FRAMES as usize}> {
        unsafe { &*self.frame_graphs.get() }
    }
}


pub struct RendererContext {
    global_resources: Arc<RwLock<GlobalResources>>,
    pub(crate) transfer_requests: TransferRequests,
    frame_buffer_size: image::Dimensions,
}

impl RendererContext {

    #[inline(always)]
    pub fn edit_resources<F>(&self, mut f: F) -> Result<(), Error>
        where
            F: FnMut(&mut GlobalResources) -> Result<(), Error>
    {
        let mut resources = self.global_resources.write().unwrap();
        f(&mut resources)
    }

    #[inline(always)]
    pub fn transfer_requests(&mut self) -> &mut TransferRequests {
        &mut self.transfer_requests
    }

    #[inline(always)]
    pub fn frame_buffer_size(&self) -> image::Dimensions {
        self.frame_buffer_size
    }

    #[inline(always)]
    pub fn buffer_size(&self, buffer: BufferID) -> Option<u64> {
        self.global_resources.read().unwrap().buffer_size(buffer)
    }
}

#[derive(Clone, Copy)]
pub struct ComputeState {
    command_buffer: vk::CommandBuffer,
    semaphore: vk::Semaphore,
    timeline_value: u64,
}

pub struct Renderer<'mem> {
    transfer_commands: GlobalVec<TransferCommands>,
    main_thread_context: ThreadContext,
    frame_states: ArrayVec<Rc<RefCell<FrameState>>, {MAX_BUFFERED_FRAMES as usize}>,
    compute_states: ArrayVec<ComputeState, {MAX_BUFFERED_FRAMES as usize}>,
    swapchain_pass_pipeline_data: SwapchainPassPipelineData,
    global_resources: Arc<RwLock<GlobalResources>>,
    device: Arc<ash::Device>,
    vulkan_context: VulkanContext<'mem>,
    _memory_layout: MemoryLayout,
    buffered_frame_count: u32,
    tmp_alloc: Arc<ArenaAlloc>,
    frame_buffer_size: image::Dimensions,
}

impl<'mem> Renderer<'mem> {

    pub(crate) fn new(
        window: &Window,
        app_name: &AppName,
        app_version: Version,
        enable_validation: bool,
        memory_layout: MemoryLayout,
        mut buffered_frame_count: u32,
        allocators: &'mem Allocators,
    ) -> Result<Self, String>
    {
        buffered_frame_count = clamp(buffered_frame_count, MIN_BUFFERED_FRAMES, MAX_BUFFERED_FRAMES);
        assert!(buffered_frame_count <= MAX_BUFFERED_FRAMES);
        allocators.realloc_frame_graphs(buffered_frame_count);
        let tmp_alloc = Arc::new(ArenaAlloc::new(memory_layout.tmp_arena_size()).ok_or(format!("failed to create tmp alloc"))?);
        let vulkan_context = VulkanContext
            ::new(
                window,
                &app_name,
                app_version,
                buffered_frame_count,
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
                    &physical_device_info,
                    memory_layout
                )
                .map_err(|e| format!("failed to create global resources ( {:?} ) ", e))?
        ));
        let swapchain_pass_pipeline_data = SwapchainPassPipelineData
            ::new(global_resources.clone(), buffered_frame_count, &tmp_alloc)
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
            buffered_frame_count,
            transfer_commands: Default::default(),
            tmp_alloc,
            frame_buffer_size: image::Dimensions::new(1, 1, 1),
        };
        let mut frame_states = ArrayVec::new();
        let mut i = 0;
        frame_states.resize_with(
            buffered_frame_count as usize,
            || {
                let device_alloc = LinearDeviceAlloc::new(
                    device.clone(),
                    memory_layout.frame_graph_device_block_size(),
                    vk::MemoryPropertyFlags::DEVICE_LOCAL,
                    vk::MemoryPropertyFlags::LAZILY_ALLOCATED | vk::MemoryPropertyFlags::PROTECTED,
                    &physical_device_info,
                    false,
                ).unwrap();
                let s = Rc::new(RefCell::new(
                    FrameState::new(device.clone(), global_resources.clone(), device_alloc).unwrap()
                ));
                i += 1;
                s
            }
        ).unwrap();
        let mut compute_command_buffers = ArrayVec::<vk::CommandBuffer, {MAX_BUFFERED_FRAMES as usize}>::new();
        let mut compute_semaphores = ArrayVec::<vk::Semaphore, {MAX_BUFFERED_FRAMES as usize}>::new();
        unsafe {
            let alloc_info = vk::CommandBufferAllocateInfo {
                s_type: vk::StructureType::COMMAND_BUFFER_ALLOCATE_INFO,
                command_pool: s.main_thread_context.compute_pool(),
                level: vk::CommandBufferLevel::PRIMARY,
                command_buffer_count: buffered_frame_count,
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
            compute_command_buffers.set_len(buffered_frame_count as usize);
            let mut timeline_info = vk::SemaphoreTypeCreateInfo {
                s_type: vk::StructureType::SEMAPHORE_TYPE_CREATE_INFO,
                semaphore_type: vk::SemaphoreType::TIMELINE,
                initial_value: 0,
                ..Default::default()
            };
            let mut semaphore_info = vk::SemaphoreCreateInfo {
                s_type: vk::StructureType::SEMAPHORE_CREATE_INFO,
                ..Default::default()
            };
            semaphore_info = semaphore_info.push_next(&mut timeline_info);
            for _ in 0..buffered_frame_count {
                let fence = device.create_semaphore(&semaphore_info, None).unwrap();
                compute_semaphores.push(fence).unwrap();
            }
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
    pub(crate) fn renderer_context(&mut self) -> RendererContext {
        RendererContext {
            global_resources: self.global_resources.clone(),
            transfer_requests: TransferRequests::new(),
            frame_buffer_size: self.frame_buffer_size,
        }
    }

    #[inline(always)]
    pub(crate) fn request_resize(&mut self, size: PhysicalSize<u32>) {
        self.vulkan_context.request_swapchain_update(self.buffered_frame_count, size);
    }

    pub(crate) fn transfer_requests<I: Interface>(
        &mut self,
        interface: Arc<RwLock<I>>,
        command_requests: TransferRequests,
    ) -> Result<GlobalVec<Option<JoinHandle<()>>>, Error>
    {

        let mut handles = GlobalVec::with_capacity(command_requests.task_count());

        let device = self.device.clone();
        let transfer_commands = &mut self.transfer_commands;
        let queue_families = self.vulkan_context.queue_family_indices();
        let interface = interface.clone();
        let global_resources = self.global_resources.clone();

        for (id, staging_block_size) in command_requests.transfer_iter() {

            let command_pool = helpers::create_command_pool(
                &device,
                vk::CommandPoolCreateFlags::TRANSIENT,
                queue_families.transfer_index(),
            )?;

            let mut vk_cmd_buffer = vk::CommandBuffer::null();
            let info = vk::CommandBufferAllocateInfo {
                s_type: vk::StructureType::COMMAND_BUFFER_ALLOCATE_INFO,
                command_pool,
                level: vk::CommandBufferLevel::PRIMARY,
                command_buffer_count: 1,
                ..Default::default()
            };
            helpers
                ::allocate_command_buffers(&device, &info, core::slice::from_mut(&mut vk_cmd_buffer))?;
            helpers
                ::begin_command_buffer(&device, vk_cmd_buffer)?;

            let alloc = LinearDeviceAlloc::new(
                self.device.clone(),
                staging_block_size,
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
                vk::MemoryPropertyFlags::from_raw(0),
                self.vulkan_context.physical_device_info(),
                true,
            ).unwrap();

            let mut commands = transfer_commands.push(TransferCommands::new(
                device.clone(),
                vk_cmd_buffer,
                command_pool,
                global_resources.clone(),
                alloc,
                queue_families.transfer_index(),
                id,
            ).unwrap());

            let handle = interface
                .write()
                .unwrap()
                .transfer_commands(id, &mut commands)
                .unwrap();

            handles.push(handle);
        }

        Ok(handles)
    }

    pub(crate) fn render<I: Interface>(
        &mut self,
        window: &Window,
        interface: Arc<RwLock<I>>,
        allocators: &'mem Allocators,
    ) -> Result<(), String>
    {
        let device = self.device.clone();
        let mut pending_transfers = Default::default();
        if !self.transfer_commands.is_empty() {
            let transfer_commands = &mut self.transfer_commands;
            pending_transfers = GlobalVec::with_capacity(transfer_commands.len());
            let mut ready_transfers = GlobalVec::with_capacity(transfer_commands.len());
            for (i, command) in transfer_commands.iter_mut().enumerate() {
                let (new, fence) = command
                    .get_fence()
                    .map_err(|e| format!("failed to create fence: {:?}", e))?;
                if new {
                    let command_buffer = command.vk_command_buffer();
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
                            self.vulkan_context.transfer_queue(),
                            &[submit_info],
                            fence,
                        ).map_err(|e| format!("failed to submit transfer commands: {:?}", e))?;
                    };
                }
                unsafe {
                    match device.wait_for_fences(&[fence], true, SwapchainContext::frame_timeout()) {
                        Ok(()) => {
                            ready_transfers.push(i);
                        },
                        Err(vk::Result::TIMEOUT) => {
                            pending_transfers.push(command.id());
                        }
                        Err(e) => {
                            return Err(format!("unexpected fence wait error: {:?}", e))
                        }
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
                .write()
                .unwrap()
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
            .write()
            .unwrap()
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
        let mut compute_submit = vk::SubmitInfo {
            s_type: vk::StructureType::SUBMIT_INFO,
            command_buffer_count: 1,
            p_command_buffers: &compute_state.command_buffer,
            wait_semaphore_count: 1,
            p_wait_semaphores: &compute_state.semaphore,
            p_wait_dst_stage_mask: &wait_stage,
            signal_semaphore_count: 1,
            p_signal_semaphores: &compute_state.semaphore,
            ..Default::default()
        };
        compute_submit = compute_submit.push_next(&mut timeline_submit);
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
        let mut frame_graph = FrameGraphImpl::new(
            self.frame_states[frame_data.frame_index as usize].clone(),
            frame_data.extent.into(),
            frame_data.command_buffer, 
            alloc,
            frame_data.frame_index,
            queue_family_indices,
        );
        interface
            .write()
            .unwrap()
            .render(&mut frame_graph, &pending_transfers)
            .map_err(|e| format!("interface failed to render ( {:?} )", e))?;
        let mut render_commands = RenderCommands::new(
            device.clone(),
            frame_data.command_buffer,
            self.global_resources.clone(),
            &self.tmp_alloc,
        );
        frame_graph
            .render(&mut *interface.write().unwrap(), &mut render_commands)
            .map_err(|e| format!("frame graph failed to render ( {:?} )", e))?;
        let frame_state = &mut self.frame_states[frame_data.frame_index as usize].borrow_mut();
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
        let wait_values = [0, compute_state.timeline_value + 1];
        let signal_values = [0, compute_state.timeline_value + 2];
        let wait_semaphores = [semaphores.wait_semaphore, compute_state.semaphore];
        let wait_stages = [semaphores.wait_stage, vk::PipelineStageFlags::COMPUTE_SHADER];
        let signal_semaphores = [semaphores.signal_semaphore, compute_state.semaphore];
        let mut timeline_submit = vk::TimelineSemaphoreSubmitInfo {
            s_type: vk::StructureType::TIMELINE_SEMAPHORE_SUBMIT_INFO,
            wait_semaphore_value_count: 2,
            p_wait_semaphore_values: wait_values.as_ptr(),
            signal_semaphore_value_count: 2,
            p_signal_semaphore_values: signal_values.as_ptr(),
            ..Default::default()
        };
        let mut submit_info = vk::SubmitInfo {
            s_type: vk::StructureType::SUBMIT_INFO,
            wait_semaphore_count: 2,
            p_wait_semaphores: wait_semaphores.as_ptr(),
            p_wait_dst_stage_mask: wait_stages.as_ptr(),
            signal_semaphore_count: 2,
            p_signal_semaphores: signal_semaphores.as_ptr(),
            command_buffer_count: 1,
            p_command_buffers: &frame_data.command_buffer,
            ..Default::default()
        };
        submit_info = submit_info.push_next(&mut timeline_submit);
        if let Err(e) = unsafe { device.queue_submit(graphics_queue, &[submit_info], fence) } {
            return Err(format!("graphics queue submit failed {:?}", e))
        }
        let present_result = swapchain_context
            .present_submit(&swapchain_loader, graphics_queue)
            .map_err(|e| format!("queue present failed {}", e))?;
        if present_result != PresentResult::Success || frame_data.suboptimal {
            self.vulkan_context.request_swapchain_update(self.buffered_frame_count, window.inner_size());
        }
        self.compute_states[frame_data.frame_index as usize].timeline_value += 2;
        Ok(())
    }

    pub(crate) fn clean_up(&mut self, allocators: &'mem Allocators) {
        println!("Nox renderer message: terminating renderer");
        unsafe {
            self.device.device_wait_idle().unwrap();
        }
        for state in &self.frame_states {
            unsafe {
                state.borrow_mut().force_clean_up();
            }
        }
        for state in &self.compute_states {
            unsafe {
                self.device.destroy_semaphore(
                    state.semaphore, None
                );
            }
        }
        self.global_resources.write().unwrap().clean_up();
        self.vulkan_context.destroy_swapchain(self.main_thread_context.graphics_pool(), &allocators);
    }
}
