use compact_str::CompactString;

use nox_mem::{
    AsRaw,
    impl_as_raw_bit_op
};

use crate::dev::has_bits;
use crate::Version;

use super::{MIN_BUFFERED_FRAMES, MAX_BUFFERED_FRAMES};

#[repr(u8)]
#[derive(AsRaw, Clone, Copy)]
pub enum Layer {
    /// Khronos validation layer. For this to be used, the Vulkan SDK needs to be installed.
    KhronosValidation = 0x1,
}

impl_as_raw_bit_op!(Layer);

#[repr(u8)]
#[derive(AsRaw, Clone, Copy)]
pub enum Extension {
    /// Enables the use of inline uniform blocks (see [`DescriptorType::InlineUniformBlock`]).
    InlineUniformBlock = 0x1,
    /// Enables Vulkan debug utils.
    DebugUtils = 0x2,
}

impl_as_raw_bit_op!(Extension);

pub struct GpuAttributes {
    pub(crate) app_name: CompactString,
    pub(crate) app_version: Version,
    pub(super) buffered_frames: u32,
    pub(super) layers: u8,
    pub(super) required_layers: u8,
    pub(super) extensions: u8,
    pub(super) optional_extensions: u8,
}

impl GpuAttributes {

    #[inline(always)]
    pub fn with_app_name(mut self, name: impl AsRef<str>) -> Self {
        self.app_name = CompactString::new(name);
        self
    }

    #[inline(always)]
    pub fn with_app_version(mut self, version: Version) -> Self {
        self.app_version = version;
        self
    }

    /// Sets how many buffered frames swapchains have (). If surface capabilities don't allow `count`
    /// buffered frames, swapchains are synchronized as if there were `count` buffered frames
    /// instead. `count` will be clamped between `MIN_BUFFERED_FRAMES` and `MAX_BUFFERED_FRAMES`.
    ///
    /// The default is 3.
    #[inline(always)]
    pub fn with_buffered_frames(mut self, count: u32) -> Self {
        self.buffered_frames = count.clamp(MIN_BUFFERED_FRAMES, MAX_BUFFERED_FRAMES);
        self
    }

    /// Adds a [`Layer`]. Layers *are not* required by default. Required layers can be added by
    /// `with_required_layers`
    #[inline(always)]
    pub fn with_layer(mut self, layer: Layer) -> Self {
        self.layers |= layer;
        self
    }

    /// Adds a required [`Layer`].
    #[inline(always)]
    pub fn with_required_layer(mut self, layer: Layer) -> Self {
        self.required_layers |= layer;
        self
    }

    /// Adds an [`Extension`]. Extensions *are* required by default. Optional extensions can be
    /// added by `with_optional_extension`.
    #[inline(always)]
    pub fn with_extension(mut self, extension: Extension) -> Self {
        self.extensions |= extension;
        self
    }

    /// Adds an optional [`Extension`].
    #[inline(always)]
    pub fn with_optional_extension(mut self, extension: Extension) -> Self {
        self.optional_extensions |= extension;
        self
    }

    /// Checks if attributes contains the given [`Layer`]. Returns `Some(true)` if the layer is found
    /// and required, `Some(false)` if it's found and not required and `None` if the layer isn't
    /// present.
    #[inline(always)]
    pub fn contains_layer(&self, layer: Layer) -> Option<bool> {
        if has_bits!(self.required_layers, layer) {
            Some(true)
        } else if has_bits!(self.layers, layer) {
            Some(false)
        } else {
            None
        }
    }
    /// Checks if attributes contains the given [`Extension`]. Returns `Some(true)` if the extension is found
    /// and required, `Some(false)` if it's found and not required and `None` if the extension isn't
    /// present.
    #[inline(always)]
    pub fn contains_extension(&self, extension: Extension) -> Option<bool> {
        if has_bits!(self.layers, extension) {
            Some(true)
        } else if has_bits!(self.optional_extensions, extension) {
            Some(false)
        } else {
            None
        }
    }
}

/// Creates default [`GpuAttributes`] with no layers or extensions.
pub fn default_attributes() -> GpuAttributes {
    GpuAttributes {
        app_name: CompactString::default(),
        app_version: Version::default(),
        buffered_frames: 3,
        layers: 0,
        required_layers: 0,
        extensions: 0,
        optional_extensions: 0,
    }
}
