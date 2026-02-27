use {
    core::{
        time::Duration,
        num::NonZeroU32,
    },
    compact_str::CompactString,
    nox_mem::{
        AsRaw,
        impl_as_raw_bit_op,
        vec::{Vec32, Vector},
        vec32,
    },
    nox_ash::vk,
    crate::{
        Version,
        dev::has_bits,
    },
};


use super::{
    MIN_BUFFERED_FRAMES, MAX_BUFFERED_FRAMES,
    MemoryLayout, ext,
    BaseDeviceFeatures,
};

#[repr(u32)]
#[derive(AsRaw, Clone, Copy)]
pub enum Layer {
    /// Khronos validation layer.
    ///
    /// For this to be used, the Vulkan SDK needs to be installed.
    KhronosValidation = 0x1,
}

#[repr(u32)]
#[derive(AsRaw, Clone, Copy)]
pub enum InstanceExtension {
    /// Enables the Vulkan debug utils instance extension.
    DebugUtils = 0x1,
}

impl_as_raw_bit_op!(Layer, InstanceExtension);

#[derive(Clone)]
pub struct GpuAttributes {
    pub(crate) app_name: CompactString,
    pub(crate) app_version: Version,
    pub(super) buffered_frames: u32,
    pub(super) layers: u32,
    pub(super) required_layers: u32,
    pub(super) instance_extensions: u32,
    pub(super) optional_instance_extensions: u32,
    pub(super) device_extensions: Vec32<ext::DeviceExtensionObj>,
    pub(super) memory_layout: MemoryLayout,
    pub(super) command_workers: u32,
    pub(super) frame_timeout: Duration,
    pub(super) required_features: BaseDeviceFeatures,
}

impl GpuAttributes {

    /// Sets the Vulkan application `name` used for debugging.
    ///
    /// The default is an empty string.
    #[inline(always)]
    pub fn with_app_name(mut self, name: impl AsRef<str>) -> Self {
        self.app_name = CompactString::new(name);
        self
    }
    
    /// Sets the Vulkan application `version` used for debugging.
    ///
    /// The default is [`Version::default()`] (1.0.0).
    #[inline(always)]
    pub fn with_app_version(mut self, version: Version) -> Self {
        self.app_version = version;
        self
    }

    /// Sets how many buffered frames swapchains have.
    ///
    /// `count` will be clamped between [`MIN_BUFFERED_FRAMES`] and [`MAX_BUFFERED_FRAMES`].
    ///
    /// If surface capabilities don't allow `count` buffered frames, swapchains are synchronized as
    /// if there were `count` buffered frames.
    ///
    /// The default buffered frame count is 3.
    #[inline(always)]
    pub fn with_buffered_frames(mut self, count: u32) -> Self {
        self.buffered_frames = count.clamp(MIN_BUFFERED_FRAMES, MAX_BUFFERED_FRAMES);
        self
    }

    /// Adds a [`Layer`].
    ///
    /// Layers *are not* required by default. Required layers can be added with `with_required_layers`.
    #[inline(always)]
    pub fn with_layer(mut self, layer: Layer) -> Self {
        self.layers |= layer;
        self
    }

    /// Adds a required [`Layer`].
    ///
    /// To add an optional layer, use `with_layer`.
    #[inline(always)]
    pub fn with_required_layer(mut self, layer: Layer) -> Self {
        self.required_layers |= layer;
        self
    }

    /// Adds an [`InstanceExtension`].
    ///
    /// Instance extensions *are* required by default. Optional extensions can be added with
    /// `with_optional_extension`.
    #[inline(always)]
    pub fn with_instance_extension(mut self, extension: InstanceExtension) -> Self {
        self.instance_extensions |= extension;
        self
    }

    /// Adds an optional [`InstanceExtension`].
    ///
    /// To add a required instance extension, use `with_extension`.
    #[inline(always)]
    pub fn with_optional_instance_extension(mut self, extension: InstanceExtension) -> Self {
        self.optional_instance_extensions |= extension;
        self
    }

    /// Adds an [`ext::DeviceExtension`].
    #[inline(always)]
    pub fn with_device_extension<Ext>(mut self, extension: Ext) -> Self
        where Ext: ext::DeviceExtension,
    {
        self.device_extensions.push(Box::new(extension).into());
        self
    }

    /// Sets the number of command pools used by the queue scheduler.
    ///
    /// The default is 8.
    #[inline(always)]
    pub fn with_command_workers(mut self, n: NonZeroU32) -> Self {
        self.command_workers = n.get();
        self
    }

    /// Sets the timeout used when waiting for work to finish per frame.
    ///
    /// The default is 2 seconds.
    #[inline(always)]
    pub fn with_frame_timeout(mut self, timeout: Duration) -> Self {
        self.frame_timeout = timeout;
        self
    }

    #[inline(always)]
    pub fn with_required_device_features(mut self, features: BaseDeviceFeatures) -> Self {
        self.required_features = features;
        self
    }

    /// Checks if the attributes contains the given [`Layer`].
    ///
    /// Returns `Some(true)` if the layer is found and required, `Some(false)` if it's found and
    /// not required and `None` if the layer isn't present.
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
    /// Checks if the attributes contains the given [`InstanceExtension`].
    ///
    /// Returns `Some(true)` if the extension is found and required, `Some(false)` if it's found
    /// and not required and `None` if the extension isn't present.
    #[inline(always)]
    pub fn contains_instance_extension(&self, extension: InstanceExtension) -> Option<bool> {
        if has_bits!(self.instance_extensions, extension) {
            Some(true)
        } else if has_bits!(self.optional_instance_extensions, extension) {
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
        instance_extensions: 0,
        optional_instance_extensions: 0,
        device_extensions: vec32![],
        memory_layout: MemoryLayout::default(),
        command_workers: 8,
        frame_timeout: Duration::from_secs(2),
        required_features: BaseDeviceFeatures::default(),
    }
}
