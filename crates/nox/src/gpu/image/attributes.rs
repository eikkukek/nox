use super::*;

#[derive(Clone, Copy)]
pub struct ImageAttributes {
    pub(super) aspects: vk::ImageAspectFlags,
    pub(super) dimensions: Dimensions,
    pub(super) format: vk::Format,
    pub(super) usage: vk::ImageUsageFlags,
    pub(super) samples: MSAA,
    pub(super) array_layers: u32,
    pub(super) mip_levels: u32,
    pub(super) cube_map: bool,
    pub(super) mutable_format: bool,
    pub(super) resolve_modes: FormatResolveModes,
}

impl ImageAttributes {

    #[inline(always)]
    pub(super) fn new() -> Self {
        Self {
            aspects: vk::ImageAspectFlags::empty(),
            dimensions: Dimensions::new(0, 0, 0),
            format: vk::Format::UNDEFINED,
            usage: Default::default(),
            samples: MSAA::X1,
            array_layers: 1,
            mip_levels: 1,
            cube_map: false,
            mutable_format: false,
            resolve_modes: Default::default(),
        }
    }

    #[inline(always)]
    pub fn with_dimensions(mut self, dimensions: Dimensions) -> Self {
        self.dimensions = dimensions;
        self
    }

    #[inline(always)]
    pub fn with_format<F: Format>(mut self, format: F, mutable: bool) -> Self {
        self.format = format.as_vk_format();
        self.aspects = format.aspects();
        self.resolve_modes = format.resolve_modes();
        self.mutable_format = mutable;
        self
    }

    #[inline(always)]
    pub fn with_usage(mut self, usage: ImageUsage) -> Self {
        self.usage |= usage.into();
        self
    }

    #[inline(always)]
    pub fn with_samples(mut self, samples: MSAA) -> Self {
        self.samples = samples;
        self
    }

    #[inline(always)]
    pub fn with_array_layers(mut self, layers: u32) -> Self { 
        self.array_layers = layers;
        self
    }

    #[inline(always)]
    pub fn with_cube_map(mut self, cube_dimensions: Option<u32>) -> Self {
        if let Some(dim) = cube_dimensions {
            self.cube_map = true;
            self.dimensions = Dimensions::new(dim, dim, 1);
            self.array_layers = self.array_layers.next_multiple_of(6);
        }
        self
    }

    #[inline(always)]
    pub fn with_mip_levels(mut self, levels: u32) -> Self {
        self.mip_levels = levels;
        self
    }

    #[inline(always)]
    pub(crate) fn build(
        self,
        vk: Arc<Vulkan>,
        alloc: &mut (impl MemoryBinder + ?Sized),
    ) -> Result<ImageMeta, dev_error::Error>
    {
        ImageMeta::new(vk, self, alloc)
    }
}

impl PartialEq for ImageAttributes {

    fn eq(&self, other: &Self) -> bool {
        self.dimensions == other.dimensions &&
        self.format == other.format &&
        self.aspects == other.aspects &&
        self.usage == other.usage &&
        self.samples == other.samples &&
        self.array_layers == other.array_layers &&
        self.mip_levels == other.mip_levels &&
        self.mutable_format == other.mutable_format
    }
}

impl Eq for ImageAttributes {}

impl Hash for ImageAttributes {

    fn hash<H: Hasher>(&self, state: &mut H) {
        self.dimensions.hash(state);
        self.format.hash(state);
        self.aspects.hash(state);
        self.usage.hash(state);
        self.samples.hash(state);
        self.array_layers.hash(state);
        self.mip_levels.hash(state);
        self.mutable_format.hash(state);
    }
}
