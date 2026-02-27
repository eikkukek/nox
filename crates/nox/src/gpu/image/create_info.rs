use super::*;

/// The create info structure for images.
pub struct ImageCreateInfo<'a> {
    pub(crate) out: &'a mut ImageId,
    pub(crate) memory_binder: ResourceBinder,
    pub(super) aspects: ImageAspectFlags,
    pub(super) dimensions: Dimensions,
    pub(super) format: vk::Format,
    pub(super) usage: ImageUsageFlags,
    pub(super) samples: MsaaSamples,
    pub(super) array_layers: u32,
    pub(super) mip_levels: u32,
    pub(super) cube_map: bool,
    pub(super) mutable_format: bool,
    pub(super) resolve_modes: FormatResolveModes,
}

impl<'a> ImageCreateInfo<'a> {

    /// Creates new [`ImageCreateInfo`].
    ///
    /// Note that the create info isn't valid until you set image dimensions and format with
    /// [`ImageCreateInfo::with_dimensions`] and [`ImageCreateInfo::with_format`].
    ///
    /// # Parameters
    /// - `out`: A mutable reference to where the [`ImageId`] of the created image will be stored.
    /// # Memory binding
    /// The default memory binder is [`ResourceBinder::default`], which always allocates a new
    /// [`vk::DeviceMemory`] object and is *not* mappable.
    ///
    /// You can specify a different memory binder by [`ImageCreateInfo::with_memory_binder`].
    #[inline(always)]
    pub fn new(out: &'a mut ImageId) -> Self {
        Self {
            out,
            memory_binder: Default::default(),
            aspects: ImageAspectFlags::empty(),
            dimensions: Dimensions::new(0, 0, 0),
            format: vk::Format::UNDEFINED,
            usage: Default::default(),
            samples: MsaaSamples::X1,
            array_layers: 1,
            mip_levels: 1,
            cube_map: false,
            mutable_format: false,
            resolve_modes: Default::default(),
        }
    }

    /// Specifies where the image gets its memory.
    ///
    /// The default is [`ResourceBinder::default`].
    #[inline(always)]
    pub fn with_memory_binder(mut self, binder: ResourceBinder) -> Self {
        self.memory_binder = binder;
        self
    }

    /// Specifies the images dimensions
    ///
    /// Note that every dimension including `depth` needs to non-zero.
    ///
    /// Setting the depth to a value other than one makes the image a 3D image, which requires the
    /// images array layers to be one.
    #[inline(always)]
    pub fn with_dimensions(mut self, dimensions: Dimensions) -> Self {
        self.dimensions = dimensions;
        self
    }

    /// Specifies the images format and whether it can be mutated when creating an image subview.
    #[inline(always)]
    pub fn with_format<F: Format>(mut self, format: F, mutable: bool) -> Self {
        self.format = format.as_vk_format();
        self.aspects = format.aspects();
        self.resolve_modes = format.resolve_modes();
        self.mutable_format = mutable;
        self
    }

    /// Specifies what the image *can* be used for.
    #[inline(always)]
    pub fn with_usage(mut self, usage: ImageUsageFlags) -> Self {
        self.usage |= usage;
        self
    }

    /// Specifies how many samples per pixel the image has in multisample anti-aliasing.
    #[inline(always)]
    pub fn with_samples(mut self, samples: MsaaSamples) -> Self {
        self.samples = samples;
        self
    }
    
    /// Specifies how many array layers the image has.
    ///
    /// Note that if this is a 3D image (depth dimension > 1), this *must* be set to one.
    #[inline(always)]
    pub fn with_array_layers(mut self, layers: u32) -> Self { 
        self.array_layers = layers;
        self
    }

    /// Specifies that the image can be used as a cube map.
    ///
    /// Requires that the width and height of the image are the same, depth is one and array layers
    /// is a multiple of six.
    ///
    /// To unset this, pass [`None`].
    #[inline(always)]
    pub fn with_cube_map(mut self, cube_dimensions: Option<u32>) -> Self {
        if let Some(dim) = cube_dimensions {
            self.cube_map = true;
            self.dimensions = Dimensions::new(dim, dim, 1);
            self.array_layers = self.array_layers.next_multiple_of(6);
        } else {
            self.cube_map = false;
        }
        self
    }

    /// Specifies how many mip levels the image has.
    #[inline(always)]
    pub fn with_mip_levels(mut self, levels: u32) -> Self {
        self.mip_levels = levels;
        self
    }

    #[inline(always)]
    pub(crate) fn build(
        &self,
        vk: Arc<Vulkan>,
        alloc: &mut (impl MemoryBinder + ?Sized),
        bind_memory_info: &mut vk::BindImageMemoryInfo<'static>,
    ) -> Result<ImageMeta, dev_error::Error>
    {
        ImageMeta::new(vk, self, alloc, bind_memory_info)
    }
}
