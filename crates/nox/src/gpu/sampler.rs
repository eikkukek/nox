use nox_ash::vk;

use crate::sync::*;

use crate::gpu::prelude::*;

struct Inner {
    handle: vk::Sampler,
    attributes: SamplerAttributes,
}

#[derive(Clone)]
pub struct Sampler {
    inner: Arc<Inner>,
}

impl Sampler {

    pub const LOD_CLAMP_NONE: f32 = vk::LOD_CLAMP_NONE;

    #[inline(always)]
    unsafe fn new(handle: vk::Sampler, attributes: SamplerAttributes) -> Self {
        Self {
            inner: Arc::new(Inner { handle, attributes })
        }
    }

    #[inline(always)]
    pub(crate) fn handle(&self) -> TransientHandle<'_, vk::Sampler> {
        TransientHandle::new(self.inner.handle)
    }

    pub fn default_attributes() -> SamplerAttributes {
        SamplerAttributes {
            mag_filter: Default::default(),
            min_filter: Default::default(),
            mip_mode: Default::default(),
            address_mode_u: Default::default(),
            address_mode_v: Default::default(),
            address_mode_w: Default::default(),
            mip_lod_bias:  0.0,
            max_anisotropy: None,
            compare_op: None,
            min_lod: 0.0,
            max_lod: 0.0,
            border_color: Default::default(),
        }
    }
}

#[derive(Clone, Copy)]
pub struct SamplerAttributes {
    pub mag_filter: Filter,
    pub min_filter: Filter,
    pub mip_mode: MipmapMode,
    pub address_mode_u: SamplerAddressMode,
    pub address_mode_v: SamplerAddressMode,
    pub address_mode_w: SamplerAddressMode,
    pub mip_lod_bias: f32,
    pub max_anisotropy: Option<f32>,
    pub compare_op: Option<CompareOp>,
    pub min_lod: f32,
    pub max_lod: f32,
    pub border_color: BorderColor,
}

impl SamplerAttributes {

    /// Specifies which magnification [`Filter`] to apply to look ups. Default is [`Filter::NEAREST`].
    ///
    /// See the Vulkan docs for more information on filtering:
    /// **<https://docs.vulkan.org/spec/latest/chapters/textures.html#textures-texel-filtering>**.
    #[inline(always)]
    pub fn with_mag_filter(mut self, filter: Filter) -> Self {
        self.mag_filter = filter;
        self
    }

    /// Specifies which minification [`Filter`] to apply to look ups. Default is [`Filter::NEAREST`].
    ///
    /// See the Vulkan docs for more information on filtering:
    /// **<https://docs.vulkan.org/spec/latest/chapters/textures.html#textures-texel-filtering>**.
    #[inline(always)]
    pub fn with_min_filter(mut self, filter: Filter) -> Self {
        self.min_filter = filter;
        self
    }

    /// Specifies which mipmap filter to apply to lookups. Default is [`MipmapMode::NEAREST`].
    ///
    /// See the Vulkan docs for more information on filtering:
    /// **<https://docs.vulkan.org/spec/latest/chapters/textures.html#textures-texel-filtering>**.
    #[inline(always)]
    pub fn with_mipmap_mode(mut self, mode: MipmapMode) -> Self {
        self.mip_mode = mode;
        self
    }

    /// Specifies the bias added to mipmap LOD calculation and bias provided by image sampling
    /// functions in SPIR-V.
    ///
    /// See the Vulkan docs for details:
    /// **<https://docs.vulkan.org/spec/latest/chapters/textures.html#textures-level-of-detail-operation>**.
    #[inline(always)]
    pub fn with_mip_lod_bias(mut self, bias: f32) -> Self {
        self.mip_lod_bias = bias;
        self
    }

    /// Enables anisotropic filtering if `max_anisotropy` is [`Some`]. The default is [`None`].
    ///
    /// See the Vulkan docs for details:
    /// **<https://docs.vulkan.org/spec/latest/chapters/textures.html#textures-texel-anisotropic-filtering>**.
    #[inline(always)]
    pub fn with_anisotropy(mut self, max_anisotropy: Option<f32>) -> Self {
        self.max_anisotropy = max_anisotropy;
        self
    }

    /// Specifies which wrapping operation is used when the coordinates of used to sample an image
    /// would be out of bounds. The default is [`AddressMode::REPEAT`] for each coordinate.
    ///
    /// See the Vulkan docs for details:
    /// **<https://docs.vulkan.org/spec/latest/chapters/textures.html#textures-wrapping-operation>**.
    #[inline(always)]
    pub fn with_address_mode(
        mut self,
        u: SamplerAddressMode,
        v: SamplerAddressMode,
        w: SamplerAddressMode,
    ) -> Self {
        self.address_mode_u = u;
        self.address_mode_v = v;
        self.address_mode_w = w;
        self
    }

    /// Specifies the value used to clamp the minimum level of detail value. The default value is
    /// `0.0`.
    ///
    /// See the Vulkan docs for details:
    /// **<https://docs.vulkan.org/spec/latest/chapters/textures.html#textures-level-of-detail-operation>**.
    #[inline(always)]
    pub fn with_min_lod(mut self, lod: f32) -> Self {
        self.min_lod = lod;
        self
    }

    /// Specifies the value used to clamp the maximum level of detail value. To disable clamping
    /// the maximum, use the [`Sampler::LOD_CLAMP_NONE`] constant. The default value is `0.0`.
    ///
    /// See the Vulkan docs for details:
    /// **<https://docs.vulkan.org/spec/latest/chapters/textures.html#textures-level-of-detail-operation>**.
    pub fn with_max_lod(mut self, lod: f32) -> Self {
        self.max_lod = lod;
        self
    }

    pub(crate) fn build(self, device: &ash::Device) -> Result<Sampler, ImageError> {
        let info = vk::SamplerCreateInfo {
            s_type: vk::StructureType::SAMPLER_CREATE_INFO,
            mag_filter: self.mag_filter.into(),
            min_filter: self.min_filter.into(),
            mipmap_mode: self.mip_mode.into(),
            address_mode_u: self.address_mode_u.into(),
            address_mode_v: self.address_mode_v.into(),
            address_mode_w: self.address_mode_w.into(),
            mip_lod_bias: self.mip_lod_bias,
            anisotropy_enable: self.max_anisotropy.is_some() as u32,
            max_anisotropy: self.max_anisotropy.unwrap_or_default(),
            compare_enable: self.compare_op.is_some() as u32,
            compare_op: self.compare_op.unwrap_or(CompareOp::Never).into(),
            min_lod: self.min_lod,
            max_lod: self.max_lod,
            border_color: self.border_color.into(),
            ..Default::default()
        };
        unsafe { 
            Ok(Sampler::new(device.create_sampler(&info, None)?, self))
        }
    }
}
