use winit::dpi::LogicalSize;

use crate::version::Version;

use super::AppName;

pub struct InitSettings {
    pub app_name: AppName,
    pub app_version: Version,
    pub window_size: LogicalSize<f32>
}

impl InitSettings {

    pub fn new(
        app_name: &str,
        app_version: Version,
        window_size: LogicalSize<f32>
    ) -> Self
    {
        InitSettings {
            app_name: AppName::from_str(app_name),
            app_version,
            window_size,
        }
    }
}
