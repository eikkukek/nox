pub mod frame_graph;
pub mod pipeline;
pub mod image;
pub mod memory_binder;
//pub mod default_binder;

mod errors;
mod memory_layout;
mod handle;
mod helpers;
mod shader_fn;
mod enums;
mod physical_device;
mod vulkan_context;
mod descriptor_pool;
mod swapchain_context;
mod thread_context;
mod frame_state;
mod linear_device_alloc;
mod global_resources;
mod commands;

mod swapchain_pass;

use std::{
    rc::Rc, sync::{Arc, RwLock}, thread::{self, JoinHandle, ThreadId}
};

use core::{
    slice,
    cell::{UnsafeCell, RefCell},
};

use ash::vk;

use fxhash::FxHashMap;

use nox_mem::vec_types::{ArrayVec, GlobalVec, Vector};

use crate::{stack_alloc::StackAlloc, utility::clamp};

use super::{
    interface::Interface,
    Version,
    string_types::{ArrayString, array_format, LargeError},
    AppName
};

pub use enums::*;
pub use errors::Error;
pub use memory_layout::MemoryLayout;
pub use handle::{Handle, RaiiHandle};
pub use image::{ImageBuilder};
pub use physical_device::QueueFamilyIndices;
pub use global_resources::*;
pub use commands::*;

//use device_allocators::DeviceAllocators;
use linear_device_alloc::LinearDeviceAlloc;
use vulkan_context::VulkanContext;
use swapchain_context::SwapchainContext;
use physical_device::PhysicalDeviceInfo;
use pipeline::{
    PipelineCache,
};
use frame_state::FrameState;
use frame_graph::FrameGraphImpl;
use swapchain_context::PresentResult;
use thread_context::ThreadContext;
use image::ImageState;

use swapchain_pass::SwapchainPassPipelineData;

use winit::{
    dpi::PhysicalSize, window::Window
};

pub type DeviceName = ArrayString<{ash::vk::MAX_PHYSICAL_DEVICE_NAME_SIZE}>;

pub const MIN_BUFFERED_FRAMES: u32 = 2;
pub const MAX_BUFFERED_FRAMES: u32 = 8;

pub struct Allocators {
    init: StackAlloc,
    swapchain: StackAlloc,
    frame_graphs: UnsafeCell<ArrayVec<StackAlloc, {MAX_BUFFERED_FRAMES as usize}>>,
    _memory_layout: MemoryLayout,
}

impl Allocators {

    pub fn new(memory_layout: MemoryLayout) -> Option<Self> {
        Some(Self {
            init: StackAlloc::new(memory_layout.init_size())?,
            swapchain: StackAlloc::new(memory_layout.swapchain_size())?,
            frame_graphs: UnsafeCell::new(Default::default()),
            _memory_layout: memory_layout,
        })
    }

    fn realloc_frame_graphs(&self, buffered_frame_count: u32) {
        assert!(buffered_frame_count <= MAX_BUFFERED_FRAMES);
        unsafe { &mut *self.frame_graphs.get() }.resize_with(
            buffered_frame_count as usize,
            || StackAlloc::new(self._memory_layout.frame_graphs_size()).unwrap()
        ).unwrap();
    }

    fn frame_graphs(&self) -> &ArrayVec<StackAlloc, {MAX_BUFFERED_FRAMES as usize}> {
        unsafe { &*self.frame_graphs.get() }
    }
}

pub struct Renderer<'mem> {
    main_thread_context: ThreadContext,
    thread_contexts: Arc<RwLock<FxHashMap<ThreadId, ThreadContext>>>,
    frame_states: ArrayVec<Rc<RefCell<FrameState>>, {MAX_BUFFERED_FRAMES as usize}>,
    swapchain_pass_pipeline_data: SwapchainPassPipelineData,
    pipeline_cache: PipelineCache,
    global_resources: Arc<RwLock<GlobalResources>>,
    device: Arc<ash::Device>,
    vulkan_context: VulkanContext<'mem>,
    memory_layout: MemoryLayout,
    buffered_frame_count: u32,
    command_requests: CommandRequests,
    transfer_commands: Arc<RwLock<GlobalVec<TransferCommandbuffer>>>,
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
    ) -> Result<Self, LargeError>
    {
        buffered_frame_count = clamp(buffered_frame_count, MIN_BUFFERED_FRAMES, MAX_BUFFERED_FRAMES);
        assert!(buffered_frame_count <= MAX_BUFFERED_FRAMES);
        allocators.realloc_frame_graphs(buffered_frame_count);
        let vulkan_context = VulkanContext
            ::new(
                window,
                &app_name,
                app_version,
                buffered_frame_count,
                enable_validation,
                allocators)
            .map_err(|e| e)?;
        let device = Arc::new(vulkan_context.device().clone());
        let main_thread_context = ThreadContext
            ::new(
                device.clone(),
                vulkan_context.queue_family_indices())
            .map_err(|e|
                array_format!("failed to create main thread context ( {:?} )", e)
            )?;
        let physical_device_info = vulkan_context.physical_device_info().clone();
        let swapchain_pass_pipeline_data = SwapchainPassPipelineData
            ::new(vulkan_context.device(), &physical_device_info, buffered_frame_count, allocators)
            .map_err(|e| array_format!("failed to create full screen pass data ( {:?} )", e))?;
        let global_resources = Arc::new(RwLock::new(
            GlobalResources::new(device.clone())
        ));
        let mut s = Self {
            pipeline_cache: PipelineCache::new(),
            main_thread_context,
            thread_contexts: Default::default(),
            vulkan_context,
            frame_states: Default::default(),
            swapchain_pass_pipeline_data,
            global_resources: global_resources.clone(),
            device: device.clone(),
            memory_layout,
            buffered_frame_count,
            command_requests: CommandRequests::new(),
            transfer_commands: Default::default(),
        };
        let mut frame_states = ArrayVec::new();
        let mut i = 0;
        frame_states.resize_with(
            buffered_frame_count as usize,
            || {
                let device_alloc = LinearDeviceAlloc::new(
                    device.clone(),
                    memory_layout.device_frame_size(),
                    vk::MemoryPropertyFlags::DEVICE_LOCAL,
                    vk::MemoryPropertyFlags::LAZILY_ALLOCATED | vk::MemoryPropertyFlags::PROTECTED,
                    &physical_device_info,
                ).unwrap();
                let s = Rc::new(RefCell::new(
                    FrameState::new(device.clone(), global_resources.clone(), device_alloc)
                ));
                i += 1;
                s
            }
        ).unwrap();
        s.frame_states = frame_states;
        Ok(s)
    }

    pub fn device_info(&self) -> &PhysicalDeviceInfo {
        self.vulkan_context.physical_device_info()
    }

    pub (crate) fn global_resources(&self) -> Arc<RwLock<GlobalResources>> {
        self.global_resources.clone()
    }

    pub(crate) fn request_resize(&mut self, size: PhysicalSize<u32>) {
        self.vulkan_context.request_swapchain_update(self.buffered_frame_count, size);
    }

    pub(crate) fn command_requests<I: Interface>(
        &mut self,
        interface: Arc<RwLock<I>>,
    ) -> GlobalVec<Option<JoinHandle<()>>>
    {
        self.command_requests.reset();
        interface.write().unwrap().command_requests(&mut self.command_requests);

        let mut handles = GlobalVec
            ::with_capacity(self.command_requests.task_count())
            .unwrap();

        let staging_alloc = Arc::new(RwLock::new(LinearDeviceAlloc::new(
            self.device.clone(),
            self.memory_layout.device_staging_size(),
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            vk::MemoryPropertyFlags::from_raw(0),
            self.vulkan_context.physical_device_info(),
        ).unwrap()));

        for (id, request) in self.command_requests.transfer_iter() {

            let device = self.device.clone();
            let capacity = request.staging_buffer_capacity;
            let alloc = staging_alloc.clone();
            let transfer_commands = self.transfer_commands.clone();
            let interface = interface.clone();
            let thread_contexts = self.thread_contexts.clone();
            let queue_families = self.vulkan_context.queue_family_indices();

            let handle = thread::spawn(move || {
                let mut thread_contexts = thread_contexts
                    .write()
                    .expect("ThreadContext lock poisoned");
                let thread_context = thread_contexts
                    .entry(thread::current().id())
                    .or_insert(ThreadContext
                        ::new(device.clone(), queue_families)
                        .unwrap()
                    );
                let vk_cmd_buffer = vk::CommandBuffer::null();
                let command_pool = thread_context.transfer_pool();
                let info = vk::CommandBufferAllocateInfo {
                    s_type: vk::StructureType::COMMAND_BUFFER_ALLOCATE_INFO,
                    command_pool,
                    level: vk::CommandBufferLevel::PRIMARY,
                    command_buffer_count: 1,
                    ..Default::default()
                };
                helpers
                    ::allocate_command_buffers(&device, &info, &mut [vk_cmd_buffer])
                    .unwrap();
                let mut command_buffer = TransferCommandbuffer::new(
                    device,
                    vk_cmd_buffer,
                    command_pool,
                    capacity,
                    alloc,
                    id,
                ).unwrap();
                interface
                    .write()
                    .expect("Interface lock poisoned")
                    .transfer_commands(id, &mut command_buffer);
                transfer_commands
                    .write()
                    .expect("Transfer commands lock poisoned")
                    .push(command_buffer)
                    .unwrap();
            });

            handles.push(Some(handle)).unwrap();
        }

        handles
    }

    pub(crate) fn render<I: Interface>(
        &mut self,
        window: &Window,
        interface: Arc<RwLock<I>>,
        allocators: &'mem Allocators,
    ) -> Result<(), LargeError>
    {
        let device = self.device.clone();
        let mut pending_transfers = Default::default();
        if !self.transfer_commands.read().unwrap().is_empty() {
            let mut transfer_commands = self.transfer_commands.write().unwrap();
            pending_transfers = GlobalVec
                ::with_capacity(transfer_commands.len())
                .unwrap();
            let mut ready_transfers = GlobalVec
                ::with_capacity(transfer_commands.len())
                .unwrap();
            for (i, command) in transfer_commands.iter_mut().enumerate() {
                let (new, fence) = command
                    .get_fence()
                    .map_err(|e| array_format!("failed to create fence: {:?}", e))?;
                if new {
                    let command_buffer = command.vk_command_buffer();
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
                        ).map_err(|e| array_format!("failed to submit transfer commands: {:?}", e))?;
                    };
                }
                unsafe {
                    match device.wait_for_fences(&[fence], true, SwapchainContext::frame_timeout()) {
                        Ok(()) => {
                            ready_transfers.push(i).unwrap();
                        },
                        Err(vk::Result::TIMEOUT) => {
                            pending_transfers.push(command.id()).unwrap();
                        }
                        Err(e) => {
                            return Err(array_format!("unexpected fence wait error: {:?}", e))
                        }
                    }
                }
            }
            for i in ready_transfers.iter().rev() {
                transfer_commands.remove(*i);
            }
        }
        else {
            self.command_requests.reset();
        }
        let swapchain_loader = self.vulkan_context.swapchain_loader().clone();
        let queue_family_indices = self.vulkan_context.queue_family_indices();
        let graphics_queue = self.vulkan_context.graphics_queue();
        let swapchain_context = self.vulkan_context
            .get_swapchain_context(
                self.main_thread_context.graphics_pool(),
                allocators)
            .map_err(|e| {
                array_format!("failed to get swapchain context ( {} )", e)
            })?;
        let frame_data = match swapchain_context
            .setup_image(&device, &swapchain_loader)
            .map_err(|e| {
                array_format!("failed to setup render image ( {} )", e)
            })? {
                Some(r) => r,
                None => return Ok(())
            };
        if let Err(e) = helpers::begin_command_buffer(&device, frame_data.command_buffer) {
            return Err(array_format!("failed to begin command buffer {:?}", e))
        }
        let pipeline = self.swapchain_pass_pipeline_data
            .get_pipeline(frame_data.format)
            .map_err(|e| array_format!("failed to get full screen pass pipeline {:?}", e))?;
        let alloc = &allocators.frame_graphs()[frame_data.frame_index as usize];
        unsafe {
            alloc.force_clear();
        }
        let mut frame_graph = FrameGraphImpl::new(
            self.frame_states[frame_data.frame_index as usize].clone(),
            frame_data.command_buffer, 
            &self.pipeline_cache,
            alloc,
            frame_data.frame_index,
            queue_family_indices,
        );
        interface
            .write()
            .unwrap()
            .render(&mut frame_graph, &pending_transfers)
            .map_err(|e| array_format!("interface failed to render ( {:?} )", e))?;
        frame_graph
            .render()
            .map_err(|e| array_format!("frame graph failed to render ( {:?} )", e))?;
        let frame_state = &self.frame_states[frame_data.frame_index as usize].borrow_mut();
        let mut image_state = frame_data.image_state;
        if let Some(render_image_id) = frame_state.render_image() {
            let graphics_queue_index = queue_family_indices.graphics_index();
            let command_buffer = frame_data.command_buffer;
            let read_image_state = ImageState::new(
                vk::AccessFlags::COLOR_ATTACHMENT_READ,
                vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                graphics_queue_index,
                vk::PipelineStageFlags::FRAGMENT_SHADER
            );
            let write_image_state = ImageState::new(
                vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
                vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                graphics_queue_index,
                vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            );
            let _guard = frame_state.cmd_memory_barrier(render_image_id, read_image_state);
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
                let (image_view, layout) = frame_state
                    .get_image_view(render_image_id)
                    .unwrap();
                device.cmd_bind_descriptor_sets(
                    command_buffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    self.swapchain_pass_pipeline_data.layout,
                    0,
                    &[self.swapchain_pass_pipeline_data.get_descriptor_set(
                        image_view,
                        layout,
                        frame_data.frame_index,
                    )],
                    Default::default(),
                );
                device.cmd_draw(command_buffer, 6, 1, 0, 0);

                device.cmd_end_rendering(command_buffer);
            }
        }
        let (submit_info, fence) = swapchain_context
            .setup_submit(
                &device,
                image_state,
                queue_family_indices.graphics_index()
            );
        if let Err(e) = unsafe { device.end_command_buffer(frame_data.command_buffer) } {
            return Err(array_format!("failed to end command buffer {:?}", e))
        }
        if let Err(e) = unsafe { device.queue_submit(graphics_queue, slice::from_ref(&submit_info), fence) } {
            return Err(array_format!("graphics queue submit failed {:?}", e))
        }
        let present_result = swapchain_context
            .present_submit(&swapchain_loader, graphics_queue)
            .map_err(|e| array_format!("queue present failed {}", e))?;
        if present_result != PresentResult::Success || frame_data.suboptimal {
            self.vulkan_context.request_swapchain_update(self.buffered_frame_count, window.inner_size());
        }
        Ok(())
    }

    pub(crate) fn clean_up(&mut self, allocators: &'mem Allocators) {
        println!("Nox renderer message: terminating renderer");
        self.vulkan_context.destroy_swapchain(self.main_thread_context.graphics_pool(), &allocators);
    }
}
