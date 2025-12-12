use nox_mem::AsRaw;

use ash::vk;

pub trait Format: Copy + AsRaw<Repr = i32> {

    fn as_vk_format(self) -> vk::Format {
        vk::Format::from_raw(self.as_raw())
    }

    fn aspects(self) -> vk::ImageAspectFlags;
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ImageFormat(pub vk::Format, pub vk::ImageAspectFlags);

impl Format for ImageFormat {

    fn as_vk_format(self) -> vk::Format {
        self.0
    }

    fn aspects(self) -> vk::ImageAspectFlags {
        self.1
    }
}

impl AsRaw for ImageFormat {

    type Repr = i32;

    fn as_raw(self) -> Self::Repr {
        self.0.as_raw()
    }
}
