use std::sync::RwLock;

use super::*;

#[derive(Clone)]
pub struct ImageBuilder {
    pub(super) device: Arc<ash::Device>,
    aspects: &'static [ImageAspect],
    dimensions: Dimensions,
    component_mapping: ComponentMapping,
    pub(crate) format: vk::Format,
    pub(crate) usage: vk::ImageUsageFlags,
    pub(crate) samples: MSAA,
    array_layers: u32,
    pub(crate) mip_levels: u32,
    cube_map: bool,
    always_cube: bool,
    mutable_format: bool,
}

impl ImageBuilder {

    #[inline(always)]
    pub(crate) fn new(device: Arc<ash::Device>) -> Self {
        Self {
            device,
            aspects: &[],
            dimensions: Dimensions::new(0, 0, 0),
            component_mapping: Default::default(),
            format: vk::Format::UNDEFINED,
            usage: Default::default(),
            samples: MSAA::X1,
            array_layers: 1,
            mip_levels: 1,
            cube_map: false,
            always_cube: false,
            mutable_format: false,
        }
    }

    #[inline(always)]
    pub fn with_dimensions(&mut self, dimensions: Dimensions) -> &mut Self {
        assert!(!dimensions.zero(), "each image dimension must be greater than 0");
        self.dimensions = dimensions;
        self
    }

    #[inline(always)]
    pub fn with_format<F: Format>(&mut self, format: F, mutable: bool) -> &mut Self {
        self.format = format.as_vk_format();
        self.aspects = format.aspects();
        self.mutable_format = mutable;
        self
    }

    #[inline(always)]
    pub fn with_usage(&mut self, usage: ImageUsage) -> &mut Self {
        self.usage |= usage.into();
        self
    }

    #[inline(always)]
    pub fn with_samples(&mut self, samples: MSAA) -> &mut Self {
        self.samples = samples;
        self
    }

    #[inline(always)]
    pub fn with_array_layers(&mut self, layers: u32) -> &mut Self {
        assert!(layers > 0, "image layers must be greater than 0");
        if self.cube_map {
            if self.always_cube {
                assert!(layers.is_multiple_of(6), "layer count of a cube map/array image must be a multiple of 6");
            }
            assert!(layers > 6, "layer count of a cube map/array image must be at least 6");
        }
        self.array_layers = layers;
        self
    }

    #[inline(always)]
    pub fn as_cube_map(&mut self, dimensions: u32, always: bool) -> &mut Self {
        self.cube_map = true;
        self.always_cube = always;
        self.array_layers = self.array_layers.next_multiple_of(6);
        self.dimensions = Dimensions::new(dimensions, dimensions, 1);
        self
    }

    #[inline(always)]
    pub fn with_mip_levels(&mut self, levels: u32) -> &mut Self {
        assert!(levels > 0, "image mip levels must be greater than 0");
        self.mip_levels = levels;
        self
    }

    #[inline(always)]
    pub fn with_component_mapping(&mut self, mapping: ComponentMapping) -> &mut Self {
        self.component_mapping = mapping;
        self
    }

    pub(crate) fn build(&mut self) -> Result<Image, Error> {
        let mut image_type = vk::ImageType::TYPE_2D;
        if self.dimensions.depth > 1 {
            assert!(self.array_layers == 1, "image layers must be 1 if depth is greater than 1");
            image_type = vk::ImageType::TYPE_3D;
        }
        assert!(self.format != vk::Format::UNDEFINED, "image format must be defined");
        let mut flags = Default::default();
        if self.mutable_format {
            flags |= vk::ImageCreateFlags::MUTABLE_FORMAT;
        }
        if self.cube_map {
            flags |= vk::ImageCreateFlags::CUBE_COMPATIBLE;
            assert!(
                self.dimensions.width == self.dimensions.height &&
                self.dimensions.depth == 1,
                "width and height of a cube map compatible image must be equal and depth must be 1",
            );
        }
        assert!(!self.dimensions.zero(), "image dimensions must not be zero");
        let create_info = vk::ImageCreateInfo {
            s_type: vk::StructureType::IMAGE_CREATE_INFO,
            flags,
            image_type,
            format: self.format,
            extent: self.dimensions.into(),
            mip_levels: self.mip_levels,
            array_layers: self.array_layers,
            samples: self.samples.into(),
            tiling: vk::ImageTiling::OPTIMAL,
            usage: self.usage,
            sharing_mode: vk::SharingMode::EXCLUSIVE,
            initial_layout: vk::ImageLayout::UNDEFINED,
            ..Default::default()
        };
        let handle = unsafe {
            (*self.device).create_image(&create_info, None)?
        };
        Ok(Image {
            handle: handle,
            memory: None,
            view: RwLock::new(None),
            device: self.device.clone(),
            subviews: Default::default(),
            state: RwLock::new(ImageState::new(
                vk::AccessFlags::NONE,
                vk::ImageLayout::UNDEFINED,
                vk::QUEUE_FAMILY_IGNORED,
                vk::PipelineStageFlags::TOP_OF_PIPE,
            )),
            component_mapping: self.component_mapping,
            properties: ImageProperties {
                dimensions: self.dimensions,
                format: self.format,
                aspect_mask: make_aspect_mask(self.aspects),
                usage: self.usage,
                samples: self.samples,
                array_layers: self.array_layers,
                mip_levels: self.mip_levels,
                create_flags: flags,
                always_cube_map: self.always_cube,
            }
        })
    }
}

impl PartialEq for ImageBuilder {

    fn eq(&self, other: &Self) -> bool {
        self.dimensions == other.dimensions &&
        self.format == other.format &&
        make_aspect_mask(self.aspects) == make_aspect_mask(other.aspects) &&
        self.usage == other.usage &&
        self.samples == other.samples &&
        self.array_layers == other.array_layers &&
        self.mip_levels == other.mip_levels &&
        self.mutable_format == other.mutable_format
    }
}

impl Eq for ImageBuilder {}

impl Hash for ImageBuilder {

    fn hash<H: Hasher>(&self, state: &mut H) {
        self.dimensions.hash(state);
        self.format.hash(state);
        make_aspect_mask(self.aspects).hash(state);
        self.usage.hash(state);
        self.samples.hash(state);
        self.array_layers.hash(state);
        self.mip_levels.hash(state);
        self.mutable_format.hash(state);
    }
}
