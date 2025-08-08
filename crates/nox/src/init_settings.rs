use crate::version::Version;

use super::AppName;

#[derive(Clone, Copy)]
pub struct InitSettings {
    pub app_name: AppName,
    pub app_version: Version,
    pub window_size: [u32; 2],
    pub enable_vulkan_validation: bool,
}

impl InitSettings {

    pub fn new(
        app_name: &str,
        app_version: Version,
        window_size: [u32; 2],
        enable_vulkan_validation: bool,
    ) -> Self
    {
        InitSettings {
            app_name: AppName::from_str(app_name),
            app_version,
            window_size,
            enable_vulkan_validation,
        }
    }
}
