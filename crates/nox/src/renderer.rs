mod errors;
mod memory_layout;
mod device_allocators;
mod handle;
mod helpers;
mod shader_fn;
mod enums;
mod physical_device;
mod vulkan_context;
mod pipeline;
mod swapchain_context;
mod thread_context;
mod frame_graph;
//mod transient_allocator;
mod linear_device_alloc;
mod buffer_allocator;
mod image_state;

use core::slice;

pub use ash;

use ash::vk;
use nox_mem::{GlobalVec, Vector};

use crate::{renderer::pipeline::graphics::WriteMask, stack_alloc::StackAlloc};

use super::{
    interface::Interface,
    Version,
    string_types::{ArrayString, array_format, LargeError},
    stack_alloc::StackGuard,
    AppName
};

pub use errors::Error;
pub use memory_layout::MemoryLayout;
pub use handle::Handle;
pub use enums::*;
pub use physical_device::QueueFamilyIndices;
pub use frame_graph::{
    FrameGraph,
    FrameGraphInit,
    PassAttachmentBuilder,
    PassPipelineBuilder,
    PassInfo,
    FrameGraphImpl,
    RenderError,
    ResourceID,
    WriteInfo,
    ResourcePool,
    Image,
    ImageResource,
};
pub use image_state::ImageState;
pub use buffer_allocator::{BufferAllocator, DeviceMemory, BufferAlloc};

use device_allocators::DeviceAllocators;
use linear_device_alloc::LinearDeviceAlloc;
use vulkan_context::VulkanContext;
use physical_device::PhysicalDeviceInfo;
use pipeline::{graphics::GraphicsPipelineInfo, PipelineCache};
use swapchain_context::PresentResult;
use thread_context::ThreadContext;

use winit::{
    dpi::PhysicalSize,
    window::Window
};

pub type DeviceName = ArrayString<{ash::vk::MAX_PHYSICAL_DEVICE_NAME_SIZE}>;

pub struct Allocators {
    init: StackAlloc,
    swapchain: StackAlloc,
}

impl Allocators {

    pub fn new(memory_layout: MemoryLayout) -> Option<Self> {
        Some(Self {
            init: StackAlloc::new(memory_layout.init_size())?,
            swapchain: StackAlloc::new(memory_layout.swapchain_size())?,
        })
    }
}

#[derive(Default)]
pub struct PipelineData {
    pipeline: vk::Pipeline,
    layout: vk::PipelineLayout,
}

pub struct Renderer<'mem> {
    main_thread_context: ThreadContext,
    _device_allocators: DeviceAllocators<'mem, GlobalVec<buffer_allocator::Block>>,
    frame_device_allocs: GlobalVec<LinearDeviceAlloc>,
    pipeline_cache: PipelineCache<'mem, StackAlloc>,
    vulkan_context: VulkanContext<'mem>,
    swapchain_pass_spirv: (shaderc::CompilationArtifact, shaderc::CompilationArtifact),
    swapchain_pass_pipelines: GlobalVec<(PipelineData, vk::Format)>,
    swapchain_pass_last_pipeline: (PipelineData, vk::Format),
    _memory_layout: MemoryLayout,
    buffered_frame_count: u32,
}

const SWAPCHAIN_PASS_VERT: &str = "
        #version 450

        layout(location = 0) out vec2 out_uv;

        vec2 positions[4] = vec2[](
            vec2(1.0, 1.0),
            vec2(-1.0, 1.0),
            vec2(-1.0, -1.0),
            vec2(1.0, -1.0),
        );

        vec2 uvs[4] = vec2[](
            vec2(0.0, 1.0),
            vec2(1.0, 1.0),
            vec2(1.0, 0.0),
            vec2(0.0, 0.0),
        );

        void main() {
            let vertex_index = gl_VertexIndex;
            out_uv = uvs[vertex_index];
            gl_Position = vec4(positions[vertex_index], 0.0, 1.0);
        }
    ";

const SWAPCHAIN_PASS_FRAG: &str = "
        #version 450

        layout(location = 0) in vec2 in_uv;

        layout(location = 0) out vec4 out_color;

        layout(set = 0, binding = 0) uniform sampler2D render_image;

        void main() {
            out_color = texture(render_image, in_uv);
        }
    ";

impl<'mem> Renderer<'mem> {

    pub fn new(
        window: &Window,
        app_name: &AppName,
        app_version: Version,
        enable_validation: bool,
        memory_layout: MemoryLayout,
        buffered_frame_count: u32,
        allocators: &'mem Allocators,
    ) -> Result<Self, LargeError> {
        let vulkan_context = VulkanContext
            ::new(
                window,
                &app_name,
                app_version,
                buffered_frame_count,
                enable_validation,
                allocators)
            .map_err(|e| e)?;
        let main_thread_context = ThreadContext
            ::new(
                vulkan_context.device(),
                vulkan_context.queue_family_indices())
            .map_err(|e|
                array_format!("failed to create main thread context ( {:?} )", e)
            )?;
        let device = vulkan_context.device().clone();
        let physical_device_info = vulkan_context.physical_device_info().clone();
        let swapchain_pass_spirv = (
            shader_fn::glsl_to_spirv(SWAPCHAIN_PASS_VERT, "", shaderc::ShaderKind::Vertex, &physical_device_info)
                .map_err(|e| array_format!("failed to compile swapchain pass vertex shader to SPIR-V {}", e))?,
            shader_fn::glsl_to_spirv(SWAPCHAIN_PASS_FRAG, "", shaderc::ShaderKind::Fragment, &physical_device_info)
                .map_err(|e| array_format!("failed to compile swapchain pass fragment shader to SPIR-V {}", e))?,
        );
        let mut frame_device_allocs = GlobalVec
            ::with_capacity(buffered_frame_count as usize)
            .map_err(|e |array_format!("global alloc failed ( {:?} )", e))?;
        for _ in 0..buffered_frame_count {
            frame_device_allocs
                .push(
                    LinearDeviceAlloc
                        ::new(
                            device.clone(),
                            memory_layout.device_frame_size(),
                            vk::MemoryPropertyFlags::DEVICE_LOCAL,
                            &physical_device_info,
                        )
                        .map_err(|e| array_format!("failed to create device allocator ( {:?} )", e))?
                )
                .map_err(|e| array_format!("global alloc failed ( {:?} )", e))?;
        }
        Ok(Self {
            vulkan_context,
            pipeline_cache: PipelineCache::new(),
            main_thread_context,
            _device_allocators: DeviceAllocators::new(
                &memory_layout,
                device,
                &physical_device_info,
                GlobalVec::with_capacity(256)
                    .map_err(|e| array_format!("global alloc failed ( {:?} )", e))?,
                GlobalVec::with_capacity(256)
                    .map_err(|e| array_format!("global alloc failed ( {:?} )", e))?,
                GlobalVec::with_capacity(256)
                    .map_err(|e| array_format!("global alloc failed ( {:?} )", e))?)
                .map_err(|e|
                    array_format!("failed to create device allocators ( {} )", e)
                )?,
            frame_device_allocs,
            swapchain_pass_spirv,
            swapchain_pass_pipelines: Default::default(),
            swapchain_pass_last_pipeline: Default::default(),
            _memory_layout: memory_layout,
            buffered_frame_count,
        })
    }

    pub fn device_info(&self) -> &PhysicalDeviceInfo {
        self.vulkan_context.physical_device_info()
    }

    pub fn request_resize(&mut self, size: PhysicalSize<u32>) {
        self.vulkan_context.request_swapchain_update(self.buffered_frame_count, size);
    }

    pub fn render<I: Interface>(
        &mut self,
        window: &Window,
        interface: &mut I,
        allocators: &'mem Allocators,
    ) -> Result<(), LargeError> {
        let device = self.vulkan_context.device().clone();
        let swapchain_loader = self.vulkan_context.swapchain_loader().clone();
        let queue_family_indices = *self.vulkan_context.queue_family_indices();
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
        if frame_data.format != self.swapchain_pass_last_pipeline.1 {

        }
        let alloc = StackGuard::new(&allocators.swapchain);
        let mut frame_graph = FrameGraphImpl::new(
            device.clone(),
            frame_data.command_buffer, 
            &alloc,
            &self.pipeline_cache,
            &self.frame_device_allocs[frame_data.frame_index as usize],
            frame_data.frame_index,
        );
        interface
            .render(&mut frame_graph, frame_data.format, queue_family_indices)
            .map_err(|e| array_format!("interface failed to render ( {:?} )", e))?;
        let mut render_image = frame_graph
            .render()
            .map_err(|e| array_format!("frame graph failed to render ( {:?} )", e))?;
        let mut image_state = frame_data.image_state;
        if let Some(_render_image) = render_image.take() {
            // do full screen pass to swapchain image and
            // update image_state
            image_state = image_state;
        }
        let (submit_info, fence) = swapchain_context
            .setup_submit(
                device.clone(),
                image_state,
                queue_family_indices.get_graphics_index()
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

    pub fn clean_up(&mut self, allocators: &'mem Allocators) {
        println!("Nox renderer message: terminating renderer");
        self.vulkan_context.destroy_swapchain(self.main_thread_context.graphics_pool(), &allocators);
    }

    fn create_swapchain_pass_pipeline(&mut self, format: vk::Format) -> Result<PipelineData, vk::Result> {
        let pipeline_info = GraphicsPipelineInfo::new().with_color_output(format, WriteMask::all(), None);
        Ok(Default::default())
    }
}
