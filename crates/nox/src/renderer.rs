pub mod frame_graph;
pub mod pipeline;
pub mod image;

mod errors;
mod memory_layout;
//mod device_allocators;
mod handle;
mod helpers;
mod shader_fn;
mod enums;
mod physical_device;
mod vulkan_context;
mod descriptor_pool;
mod swapchain_context;
mod thread_context;
//mod transient_allocator;
mod linear_device_alloc;
mod buffer_allocator;

use core::slice;

pub use ash;

use ash::vk;
use nox_mem::{slice, ArrayVec, GlobalVec, Vector};

use crate::{stack_alloc::StackAlloc, utility::clamp};

use super::{
    interface::Interface,
    Version,
    string_types::{ArrayString, array_format, LargeError},
    stack_alloc::StackGuard,
    AppName
};

pub use enums::*;
pub use errors::Error;
pub use memory_layout::MemoryLayout;
pub use handle::{Handle, RaiiHandle};
pub use image::{Image, ImageBuilder};
pub use physical_device::QueueFamilyIndices;
pub use image::ImageState;
pub use buffer_allocator::{BufferAllocator, DeviceMemory, BufferAlloc};

//use device_allocators::DeviceAllocators;
use linear_device_alloc::LinearDeviceAlloc;
use vulkan_context::VulkanContext;
use physical_device::PhysicalDeviceInfo;
use descriptor_pool::{
    DescriptorPoolInfo,
    DescriptorPoolSize,
};
use pipeline::{
    DescriptorType,
    ShaderStage,
    DescriptorBindingInfo,
    DescriptorSetLayoutInfo,
    GraphicsPipelineInfo,
    PipelineLayoutInfo,
    PipelineCache,
    graphics::WriteMask,
};
use frame_graph::FrameGraphImpl;
use swapchain_context::PresentResult;
use thread_context::ThreadContext;

use winit::{
    dpi::PhysicalSize, window::Window
};

pub type DeviceName = ArrayString<{ash::vk::MAX_PHYSICAL_DEVICE_NAME_SIZE}>;

pub const MIN_BUFFERED_FRAMES: u32 = 2;
pub const MAX_BUFFERED_FRAMES: u32 = 8;

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

pub struct SwapchainPassPipelineData {
    device: *const ash::Device,
    layout: vk::PipelineLayout,
    set_layout: vk::DescriptorSetLayout,
    descriptor_pool: vk::DescriptorPool,
    descriptor_sets: ArrayVec<vk::DescriptorSet, {MAX_BUFFERED_FRAMES as usize}>,
    sampler: vk::Sampler,
    pipelines: GlobalVec<(vk::Pipeline, vk::Format)>,
    last_pipeline: (vk::Pipeline, vk::Format),
    modules: (vk::ShaderModule, vk::ShaderModule),
}

impl SwapchainPassPipelineData {

    fn new(
        device: &ash::Device,
        physical_device_info: &PhysicalDeviceInfo,
        buffered_frame_count: u32,
        allocators: &Allocators,
    ) -> Result<Self, Error>
    {
        assert!(buffered_frame_count <= MAX_BUFFERED_FRAMES);
        let spirv = (
            shader_fn::glsl_to_spirv(Self::vertex_shader_input(), "", shaderc::ShaderKind::Vertex, &physical_device_info)?,
            shader_fn::glsl_to_spirv(Self::fragment_shader_input(), "", shaderc::ShaderKind::Fragment, &physical_device_info)?,
        );

        let modules = (
            pipeline::create_shader_module(&device, spirv.0.as_binary())?,
            pipeline::create_shader_module(&device, spirv.0.as_binary())?,
        );

        let set_layout = RaiiHandle::new(
            DescriptorSetLayoutInfo::new(1)
                .with_binding(DescriptorBindingInfo::new(
                    0,
                    DescriptorType::CombinedImageSampler,
                    1,
                    ShaderStage::Fragment))
                .build(&device)?,
            |handle| unsafe { device.destroy_descriptor_set_layout(handle, None) }
        );

        let layout = RaiiHandle::new(
            PipelineLayoutInfo
                ::new(1, 0)
                .with_set_layout(*set_layout)
                .build(&device)?,
            |handle| unsafe { device.destroy_pipeline_layout(handle, None) }
        );

        let stack_guard = StackGuard::new(&allocators.init);

        let descriptor_pool = RaiiHandle::new(
            DescriptorPoolInfo
                ::new(1, MAX_BUFFERED_FRAMES, &stack_guard)?
                .with_pool_sizes(slice![DescriptorPoolSize::new(DescriptorType::CombinedImageSampler, MAX_BUFFERED_FRAMES)])
                .build_raw(&device)?,
            |handle| unsafe { device.destroy_descriptor_pool(handle, None) }
        );

        let mut descriptor_sets = ArrayVec::new();
        let set_layouts = [*set_layout; MAX_BUFFERED_FRAMES as usize];

        let result = unsafe { (device.fp_v1_0().allocate_descriptor_sets)(
            device.handle(),
            &vk::DescriptorSetAllocateInfo {
                s_type: vk::StructureType::DESCRIPTOR_SET_ALLOCATE_INFO,
                descriptor_pool: *descriptor_pool,
                descriptor_set_count: buffered_frame_count,
                p_set_layouts: set_layouts.as_ptr(),
                ..Default::default()
            },
            descriptor_sets.as_mut_ptr()
        )};

        if result != vk::Result::SUCCESS {
            return Err(result.into())
        }

        unsafe {
            descriptor_sets.set_len(buffered_frame_count as usize);
        }

        let sampler = unsafe { device.create_sampler(
            &vk::SamplerCreateInfo {
                s_type: vk::StructureType::SAMPLER_CREATE_INFO,
                address_mode_u: vk::SamplerAddressMode::CLAMP_TO_BORDER,
                address_mode_v: vk::SamplerAddressMode::CLAMP_TO_BORDER,
                address_mode_w: vk::SamplerAddressMode::CLAMP_TO_BORDER,
                ..Default::default()
            },
            None
        )}?;

        Ok(Self {
            device: device,
            layout: layout.into_inner(),
            set_layout: set_layout.into_inner(),
            descriptor_pool: descriptor_pool.into_inner(),
            descriptor_sets,
            sampler,
            pipelines: GlobalVec::new(),
            last_pipeline: Default::default(),
            modules,
        })
    }

    fn update_buffered_frame_count(&mut self, buffered_frame_count: u32) -> Result<(), Error> {
        unsafe {

            let device = &*self.device;

            device.reset_descriptor_pool(self.descriptor_pool, Default::default())?;

            let set_layouts = [self.set_layout; MAX_BUFFERED_FRAMES as usize];

            let result = (device.fp_v1_0().allocate_descriptor_sets)(
                device.handle(),
                &vk::DescriptorSetAllocateInfo {
                    s_type: vk::StructureType::DESCRIPTOR_SET_ALLOCATE_INFO,
                    descriptor_pool: self.descriptor_pool,
                    descriptor_set_count: buffered_frame_count,
                    p_set_layouts: set_layouts.as_ptr(),
                    ..Default::default()
                },
                self.descriptor_sets.as_mut_ptr()
            );

            if result != vk::Result::SUCCESS {
                return Err(result.into())
            }

            self.descriptor_sets.set_len(buffered_frame_count as usize);

            Ok(())
        }
    }

    fn get_pipeline(&mut self, format: vk::Format) -> Result<vk::Pipeline, Error> {
        if format == self.last_pipeline.1 {
            return Ok(self.last_pipeline.0)
        }
        if let Some(pipeline) = self.pipelines.iter().find(|p| p.1 == format) {
            self.last_pipeline = *pipeline;
            return Ok(pipeline.0)
        }
        let info = GraphicsPipelineInfo
            ::new()
            .with_color_output(format, WriteMask::all(), None);
        let shader_entry = std::ffi::CString::new("main").unwrap();
        let stages = [GlobalVec::from(slice![
            vk::PipelineShaderStageCreateInfo {
                s_type: vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
                stage: vk::ShaderStageFlags::VERTEX,
                module: self.modules.0,
                p_name: shader_entry.as_ptr(),
                ..Default::default()
            },
            vk::PipelineShaderStageCreateInfo {
                s_type: vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
                stage: vk::ShaderStageFlags::FRAGMENT,
                module: self.modules.1,
                p_name: shader_entry.as_ptr(),
                ..Default::default()
            },
        ])];
        let pipeline = self.pipelines.push((pipeline::create_transient_graphics_pipelines(
            unsafe { &*self.device },
            slice![(info.as_create_info(), 0, 0, 0)],
            slice![vk::PipelineVertexInputStateCreateInfo {
                s_type: vk::StructureType::PIPELINE_VERTEX_INPUT_STATE_CREATE_INFO,
                ..Default::default()
            }],
            stages.as_slice(),
            slice![self.layout]
            )?[0], format)
        ).unwrap().0;
        Ok(pipeline)
    }

    pub fn get_descriptor_set(
        &mut self,
        image_view: vk::ImageView,
        image_layout: vk::ImageLayout,
        frame_index: u32,
    ) -> vk::DescriptorSet
    {
        let image_info = vk::DescriptorImageInfo {
            sampler: self.sampler,
            image_view,
            image_layout,
        };
        let descriptor_set = self.descriptor_sets[frame_index as usize];
        let write_descriptor_set = vk::WriteDescriptorSet {
            s_type: vk::StructureType::WRITE_DESCRIPTOR_SET,
            dst_set: descriptor_set,
            dst_binding: 0,
            dst_array_element: 0,
            descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            descriptor_count: 1,
            p_image_info: &image_info,
            ..Default::default()
        };
        unsafe {
            (*self.device).update_descriptor_sets(slice![write_descriptor_set], Default::default());
        }
        descriptor_set
    }

    const fn vertex_shader_input() -> &'static str {
        "
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
        "
    }

    const fn fragment_shader_input() -> &'static str {
        "
            #version 450

            layout(location = 0) in vec2 in_uv;

            layout(location = 0) out vec4 out_color;

            layout(set = 0, binding = 0) uniform sampler2D render_image;

            void main() {
                out_color = texture(render_image, in_uv);
            }
        "
    }
}

impl Drop for SwapchainPassPipelineData {

    fn drop(&mut self) {
        unsafe {
            let device = &*self.device;
            device.destroy_shader_module(self.modules.0, None);
            device.destroy_shader_module(self.modules.1, None);
            for pipeline in &self.pipelines {
                device.destroy_pipeline(pipeline.0, None);
            }
            device.destroy_pipeline_layout(self.layout, None);
            device.destroy_descriptor_pool(self.descriptor_pool, None);
            device.destroy_descriptor_set_layout(self.set_layout, None);
        }
    }
}

pub struct ID {
    uid: u64,
}

pub struct Renderer<'mem> {
    main_thread_context: ThreadContext,
    frame_device_allocs: GlobalVec<LinearDeviceAlloc>,
    swapchain_pass_pipeline_data: SwapchainPassPipelineData,
    pipeline_cache: PipelineCache<'mem, StackAlloc>,
    images: GlobalVec<Image>,
    vulkan_context: VulkanContext<'mem>,
    _memory_layout: MemoryLayout,
    buffered_frame_count: u32,
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
                &vulkan_context.device(),
                vulkan_context.queue_family_indices())
            .map_err(|e|
                array_format!("failed to create main thread context ( {:?} )", e)
            )?;
        let physical_device_info = vulkan_context.physical_device_info().clone();
        let mut frame_device_allocs = GlobalVec
            ::with_capacity(buffered_frame_count as usize)
            .map_err(|e |array_format!("global alloc failed ( {:?} )", e))?;
        for _ in 0..buffered_frame_count {
            frame_device_allocs
                .push(
                    LinearDeviceAlloc
                        ::new(
                            vulkan_context.device(),
                            memory_layout.device_frame_size(),
                            vk::MemoryPropertyFlags::DEVICE_LOCAL,
                            &physical_device_info,
                        )
                        .map_err(|e| array_format!("failed to create device allocator ( {:?} )", e))?
                )
                .map_err(|e| array_format!("global alloc failed ( {:?} )", e))?;
        }
        let swapchain_pass_pipeline_data = SwapchainPassPipelineData
            ::new(vulkan_context.device(), &physical_device_info, buffered_frame_count, allocators)
            .map_err(|e| array_format!("failed to create full screen pass data ( {:?} )", e))?;
        Ok(Self {
            pipeline_cache: PipelineCache::new(),
            main_thread_context,
            images: GlobalVec::new(),
            vulkan_context,
            frame_device_allocs,
            swapchain_pass_pipeline_data,
            _memory_layout: memory_layout,
            buffered_frame_count,
        })
    }

    pub fn device_info(&self) -> &PhysicalDeviceInfo {
        self.vulkan_context.physical_device_info()
    }

    pub fn create_image<F: FnMut(&mut ImageBuilder)>(
        &mut self,
        mut f: F
    ) -> Result<ID, Error> {
        let mut builder = ImageBuilder::new(self);
        f(&mut builder);
        self.images.push(builder.build()?).unwrap();
        Ok(ID { uid: (self.images.len() - 1) as u64 })
    }

    pub(crate) fn device(&self) -> &ash::Device {
        self.vulkan_context.device()
    }

    pub(crate) fn request_resize(&mut self, size: PhysicalSize<u32>) {
        self.vulkan_context.request_swapchain_update(self.buffered_frame_count, size);
    }

    pub(crate) fn render<I: Interface>(
        &mut self,
        window: &Window,
        interface: &mut I,
        allocators: &'mem Allocators,
    ) -> Result<(), LargeError>
    {
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
        let pipeline = self.swapchain_pass_pipeline_data
            .get_pipeline(frame_data.format)
            .map_err(|e| array_format!("failed to get full screen pass pipeline {:?}", e))?;
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
        let render_image = frame_graph
            .render()
            .map_err(|e| array_format!("frame graph failed to render ( {:?} )", e))?;
        let mut image_state = frame_data.image_state;
        if let Some(render_image) = render_image {
            let graphics_queue_index = queue_family_indices.get_graphics_index();
            let command_buffer = frame_data.command_buffer;
            if let Some(image_resource) = &mut render_image.resource() {
                let read_image_state = ImageState::new(
                    vk::AccessFlags::COLOR_ATTACHMENT_READ,
                    vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                    graphics_queue_index,
                    vk::PipelineStageFlags::FRAGMENT_SHADER
                );
                let write_image_state = ImageState::new(
                    vk::AccessFlags::COLOR_ATTACHMENT_READ,
                    vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                    graphics_queue_index,
                    vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
                );
                if image_resource.state() != read_image_state {
                    let memory_barrier = image_resource
                        .state()
                        .to_memory_barrier(
                            image_resource.image(),
                            &read_image_state,
                            *render_image.subresource_range()
                        );
                    unsafe {
                        device.cmd_pipeline_barrier(
                            command_buffer,
                            image_resource.state().pipeline_stage,
                            read_image_state.pipeline_stage,
                            Default::default(), Default::default(), Default::default(),
                            slice![memory_barrier]
                        );
                    }
                    unsafe {
                        image_resource.set_state(read_image_state);
                    }
                }
                let memory_barrier = image_state.to_memory_barrier(
                    frame_data.image,
                    &write_image_state,
                    SwapchainContext::image_subresource_range(),
                );
                unsafe {
                    device.cmd_pipeline_barrier(
                        command_buffer,
                        image_state.pipeline_stage,
                        write_image_state.pipeline_stage,
                        Default::default(), Default::default(), Default::default(),
                        slice!(memory_barrier)
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
                    device.cmd_begin_rendering(command_buffer, &rendering_info);
                }
            }
            unsafe {
                device.cmd_bind_pipeline(command_buffer, vk::PipelineBindPoint::GRAPHICS, pipeline);
                device.cmd_bind_descriptor_sets(
                    command_buffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    self.swapchain_pass_pipeline_data.layout,
                    0,
                    slice![self.swapchain_pass_pipeline_data.get_descriptor_set(
                        frame_data.image_view,
                        image_state.layout,
                        frame_data.frame_index,
                    )],
                    Default::default(),
                );
                device.cmd_draw(command_buffer, 4, 1, 0, 0);
            }
            // do full screen pass to swapchain image and
            // update image_state
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

    pub(crate) fn clean_up(&mut self, allocators: &'mem Allocators) {
        println!("Nox renderer message: terminating renderer");
        self.vulkan_context.destroy_swapchain(self.main_thread_context.graphics_pool(), &allocators);
    }
}
