use std::sync::Arc;

use ash::vk;

use nox_mem::vec_types::{ArrayVec, GlobalVec, Vector};

use crate::renderer::{
    *,
    shader::Shader,
};

#[derive(Clone)]
pub(crate) struct PipelineLayout {
    device: Arc<ash::Device>,
    handle: vk::PipelineLayout,
    pipeline_descriptor_sets: GlobalVec<(GlobalVec<Option<vk::DescriptorType>>, vk::DescriptorSetLayout)>,
    push_constant_ranges: GlobalVec<vk::PushConstantRange>,
    shader_ids: GlobalVec<ShaderId>,
}

impl PipelineLayout {

    pub fn new<const SHADER_COUNT: usize>(
        device: Arc<ash::Device>,
        shader_ids: [ShaderId; SHADER_COUNT],
        global_resources: &GlobalResources,
    ) -> Result<Self, Error>
    {
        let mut set_infos = GlobalVec::<GlobalVec<vk::DescriptorSetLayoutBinding>>::new();
        let mut push_constants = GlobalVec::new();
        let mut shaders = ArrayVec::<&Shader, SHADER_COUNT>::new();
        for id in shader_ids {
            shaders.push(global_resources.get_shader(id)?).unwrap();
        }
        for shader in &shaders {
            for uniform in shader.uniforms() {
                if uniform.set >= set_infos.len() as u32 {
                    set_infos.resize(uniform.set as usize + 1, GlobalVec::new());
                }
                let bindings = &mut set_infos[uniform.set as usize];
                if uniform.binding >= bindings.len() as u32 {
                    let mut binding = bindings.len() as u32;
                    bindings.resize_with(
                        uniform.binding as usize + 1,
                        || {
                            let s = vk::DescriptorSetLayoutBinding {
                                binding,
                                ..Default::default()
                            };
                            binding += 1;
                            s
                        }
                    ).unwrap();
                }
                bindings[uniform.binding as usize] = uniform.to_vk(&[]);
            }
            for &push_constant in shader.push_constants() {
                push_constants.push(push_constant.into());
            }
        }
        let mut set_layouts = RaiiHandle::new(
            GlobalVec::with_capacity(set_infos.capacity()),
            |v| { unsafe {
                for layout in &v {
                    device.destroy_descriptor_set_layout(*layout, None);
                }
            }}
        );
        for info in &set_infos {
            let binding_count = info.len() as u32;
            let mut flags = GlobalVec::with_capacity(info.len());
            for _ in 0..binding_count {
                flags.push(
                    vk::DescriptorBindingFlags::UPDATE_AFTER_BIND |
                    vk::DescriptorBindingFlags::UPDATE_UNUSED_WHILE_PENDING
                );
            }
            let binding_flags = vk::DescriptorSetLayoutBindingFlagsCreateInfo {
                s_type: vk::StructureType::DESCRIPTOR_SET_LAYOUT_BINDING_FLAGS_CREATE_INFO,
                binding_count: binding_count,
                p_binding_flags: flags.as_ptr(),
                ..Default::default()

            };
            let create_info = vk::DescriptorSetLayoutCreateInfo {
                s_type: vk::StructureType::DESCRIPTOR_SET_LAYOUT_CREATE_INFO,
                p_next: &binding_flags as *const _ as *const _,
                flags: vk::DescriptorSetLayoutCreateFlags::UPDATE_AFTER_BIND_POOL,
                binding_count: binding_count,
                p_bindings: info.as_ptr(),
                ..Default::default()
            };
            unsafe {
                set_layouts
                    .push(device.create_descriptor_set_layout(&create_info, None)?);
            }
        }
        let info = vk::PipelineLayoutCreateInfo {
            s_type: vk::StructureType::PIPELINE_LAYOUT_CREATE_INFO,
            set_layout_count: set_layouts.len() as u32,
            p_set_layouts: set_layouts.as_ptr(),
            push_constant_range_count: push_constants.len() as u32,
            p_push_constant_ranges: push_constants.as_ptr(),
            ..Default::default()
        };
        let handle = unsafe {
            device.create_pipeline_layout(&info, None)?
        };
        let set_layouts = set_layouts.into_inner();
        let mut pipeline_descriptor_sets = GlobalVec::with_capacity(set_layouts.len());
        for (i, layout) in set_layouts.iter().enumerate() {
            let set_info = &set_infos[i];
            let mut types = GlobalVec
                ::with_capacity(set_info.len());
            for binding in set_info {
                types.push(
                    if binding.stage_flags.is_empty() { None }
                    else { Some(binding.descriptor_type) }
                ).unwrap();
            }
            pipeline_descriptor_sets.push((
                types,
                *layout,
            ));
        }
        Ok(Self {
            pipeline_descriptor_sets,
            device,
            handle,
            push_constant_ranges: push_constants,
            shader_ids: GlobalVec::from(shader_ids.as_slice()),
        })
    }

    pub fn handle(&self) -> vk::PipelineLayout {
        self.handle
    }

    pub fn pipeline_descriptor_sets(&self) -> &[(GlobalVec<Option<vk::DescriptorType>>, vk::DescriptorSetLayout)] {
        &self.pipeline_descriptor_sets
    }

    pub fn push_constant_ranges(&self) -> &[vk::PushConstantRange] {
        &self.push_constant_ranges
    }

    pub fn shader_ids(&self) -> &[ShaderId] {
        &self.shader_ids
    }
}

impl Drop for PipelineLayout {

    fn drop(&mut self) {
        unsafe {
            self.device.destroy_pipeline_layout(self.handle(), None);
            for (_, set) in &self.pipeline_descriptor_sets {
                self.device.destroy_descriptor_set_layout(*set, None);
            }
            self.pipeline_descriptor_sets.clear();
        }
    }
}
