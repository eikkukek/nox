use ash::vk;

use nox_mem::{Vector, vec_types::{ArrayVec, GlobalVec}, slice};

use crate::stack_alloc::StackGuard;

use super::{
    Error,
    MAX_BUFFERED_FRAMES,
    PhysicalDeviceInfo,
    Allocators, 
    RaiiHandle,
    pipeline::{
        self,
        DescriptorSetLayoutInfo,
        PipelineLayoutInfo,
        DescriptorType,
        DescriptorBindingInfo,
        ShaderStage,
        GraphicsPipelineInfo,
        WriteMask,
    },
    descriptor_pool::{
        DescriptorPoolInfo,
        DescriptorPoolSize,
    },
    shader_fn,
};

pub(crate) struct SwapchainPassPipelineData {
    device: *const ash::Device,
    pub(crate) layout: vk::PipelineLayout,
    set_layout: vk::DescriptorSetLayout,
    descriptor_pool: vk::DescriptorPool,
    descriptor_sets: ArrayVec<vk::DescriptorSet, {MAX_BUFFERED_FRAMES as usize}>,
    sampler: vk::Sampler,
    pipelines: GlobalVec<(vk::Pipeline, vk::Format)>,
    last_pipeline: (vk::Pipeline, vk::Format),
    modules: (vk::ShaderModule, vk::ShaderModule),
}

impl SwapchainPassPipelineData {

    pub fn new(
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
            pipeline::create_shader_module(&device, spirv.1.as_binary())?,
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
                .with_pool_sizes(&[DescriptorPoolSize::new(DescriptorType::CombinedImageSampler, MAX_BUFFERED_FRAMES)])
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

    pub fn _update_buffered_frame_count(&mut self, buffered_frame_count: u32) -> Result<(), Error> {
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

    pub fn get_pipeline(&mut self, format: vk::Format) -> Result<vk::Pipeline, Error> {
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
            &[(info.as_create_info(), 0, 0, 0)],
            &[vk::PipelineVertexInputStateCreateInfo {
                s_type: vk::StructureType::PIPELINE_VERTEX_INPUT_STATE_CREATE_INFO,
                ..Default::default()
            }],
            stages.as_slice(),
            &[self.layout]
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
            (*self.device).update_descriptor_sets(&[write_descriptor_set], Default::default());
        }
        descriptor_set
    }

    const fn vertex_shader_input() -> &'static str {
        "
            #version 450

            layout(location = 0) out vec2 out_uv;

            vec2 positions[6] = vec2[](
                vec2(1.0, 1.0),
                vec2(-1.0, 1.0),
                vec2(-1.0, -1.0),
                vec2(1.0, -1.0),
                vec2(1.0, 1.0),
                vec2(-1.0, -1.0)
            );

            vec2 uvs[6] = vec2[](
                vec2(0.0, 1.0),
                vec2(1.0, 1.0),
                vec2(1.0, 0.0),
                vec2(0.0, 0.0),
                vec2(0.0, 1.0),
                vec2(1.0, 0.0)
            );

            void main() {
                int vertex_index = gl_VertexIndex;
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
            device.destroy_sampler(self.sampler, None);
        }
    }
}
