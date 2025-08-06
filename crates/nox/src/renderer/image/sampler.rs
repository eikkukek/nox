use super::*;

use crate::renderer::CompareOp;

#[derive(Clone, Copy)]
pub struct SamplerBuilder {
    pub(crate) mag_filter: Filter,
    pub(crate) min_filter: Filter,
    pub(crate) mip_mode: MipMode,
    pub(crate) address_mode_u: AddressMode,
    pub(crate) address_mode_v: AddressMode,
    pub(crate) address_mode_w: AddressMode,
    pub(crate) mip_lod_bias: f32,
    pub(crate) anisotropy_enable: bool,
    pub(crate) max_anisotropy: f32,
    pub(crate) compare_enable: bool,
    pub(crate) compare_op: CompareOp,
    pub(crate) min_lod: f32,
    pub(crate) max_lod: f32,
    pub(crate) border_color: BorderColor,
}

impl SamplerBuilder {

    pub fn new() -> Self {
        Self {
            mag_filter: Default::default(),
            min_filter: Default::default(),
            mip_mode: Default::default(),
            address_mode_u: Default::default(),
            address_mode_v: Default::default(),
            address_mode_w: Default::default(),
            mip_lod_bias:  0.0,
            anisotropy_enable: false,
            max_anisotropy: 0.0,
            compare_enable: false,
            compare_op: CompareOp::Never,
            min_lod: 0.0,
            max_lod: 0.0,
            border_color: Default::default(),
        }
    }

    pub(crate) fn build(self, device: &ash::Device) -> Result<vk::Sampler, Error> {
        let info = vk::SamplerCreateInfo {
            s_type: vk::StructureType::SAMPLER_CREATE_INFO,
            mag_filter: self.mag_filter.into(),
            min_filter: self.min_filter.into(),
            mipmap_mode: self.mip_mode.into(),
            address_mode_u: self.address_mode_u.into(),
            address_mode_v: self.address_mode_v.into(),
            address_mode_w: self.address_mode_w.into(),
            mip_lod_bias: self.mip_lod_bias.into(),
            anisotropy_enable: self.anisotropy_enable as u32,
            max_anisotropy: self.max_anisotropy,
            compare_enable: self.compare_enable as u32,
            compare_op: self.compare_op.into(),
            min_lod: self.min_lod,
            max_lod: self.max_lod,
            border_color: self.border_color.into(),
            ..Default::default()
        };
        unsafe {
            device.create_sampler(&info, None).map_err(Into::into)
        }
    }
}
