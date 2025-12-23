use compact_str::CompactString;

use crate::version::Version;

pub struct Attributes {
    pub(crate) app_name: CompactString,
    pub(crate) app_version: Version,
    pub(crate) close_on_no_windows: bool,
    pub(crate) vulkan_validation: bool,
}

impl Attributes {

    #[inline(always)]
    pub(crate) fn new() -> Self
    {
        Attributes {
            app_name: Default::default(),
            app_version: Default::default(),
            close_on_no_windows: true,
            vulkan_validation: false,
        }
    }

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

    /// Sets whether nox terminates when there are no active windows.
    ///
    /// Default is `true`.
    #[inline(always)]
    pub fn with_close_on_no_windows(mut self, value: bool) -> Self {
        self.close_on_no_windows = value;
        self
    }

    #[inline(always)]
    pub fn with_vulkan_validation(mut self, value: bool) -> Self {
        self.vulkan_validation = value;
        self
    }
}
