use crate::gpu;

pub struct Attributes {
    pub(crate) gpu_attributes: gpu::GpuAttributes,
    pub(crate) close_on_no_windows: bool,
}

impl Attributes {

    #[inline(always)]
    pub(crate) fn new() -> Self
    {
        Attributes {
            close_on_no_windows: true,
            gpu_attributes: gpu::default_attributes(),
        }
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
    pub fn with_gpu_attributes(mut self, attributes: gpu::GpuAttributes) -> Self {
        self.gpu_attributes = attributes;
        self
    }
}
