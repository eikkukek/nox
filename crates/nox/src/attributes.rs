use compact_str::CompactString;

use crate::version::Version;
use crate::win;

pub struct Attributes {
    pub app_name: CompactString,
    pub window_attributes: win::WindowAttributes,
    pub app_version: Version,
    pub vulkan_validation: bool,
}

impl Attributes {

    #[inline(always)]
    pub fn new(
        app_name: impl AsRef<str>,
        window_attributes: win::WindowAttributes,
    ) -> Self
    {
        Attributes {
            app_name: CompactString::new(app_name),
            window_attributes,
            app_version: Default::default(),
            vulkan_validation: false,
        }
    }

    pub fn with_app_version(mut self, version: Version) -> Self {
        self.app_version = version;
        self
    }

    pub fn with_vulkan_validation(mut self, value: bool) -> Self {
        self.vulkan_validation = value;
        self
    }
}

pub(crate) struct JustNoxAttributes {
    pub app_name: CompactString,
    pub app_version: Version,
    pub vulkan_validation: bool,
}

pub(crate) struct AttributesInternal {
    attr: Option<Attributes>,
}

impl AttributesInternal {

    #[inline(always)]
    pub fn new(attr: Attributes) -> Self {
        Self {
            attr: Some(attr),
        }
    }

    #[inline(always)]
    pub fn separate(&mut self) -> (JustNoxAttributes, winit::window::WindowAttributes) {
        let attr = self.attr.take().unwrap();
        (
            JustNoxAttributes {
                app_name: attr.app_name,
                app_version: attr.app_version,
                vulkan_validation: attr.vulkan_validation,
            },
            attr.window_attributes.to_winit_attr(),
        )
    }
}
