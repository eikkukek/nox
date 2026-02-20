use crate::gpu;

pub struct Attributes {
    pub(crate) gpu_attributes: gpu::GpuAttributes,
}

impl Attributes {

    #[inline(always)]
    pub(crate) fn new() -> Self
    {
        Attributes {
            gpu_attributes: gpu::default_attributes(),
        }
    }

    #[inline(always)]
    pub fn with_gpu_attributes(mut self, attributes: gpu::GpuAttributes) -> Self {
        self.gpu_attributes = attributes;
        self
    }
}
