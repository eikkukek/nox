use ash::vk;
use nox_mem::AsRaw;

#[repr(u32)]
#[derive(Default, Clone, Copy, AsRaw)]
pub enum MSAA {
    #[default]
    X1 = vk::SampleCountFlags::TYPE_1.as_raw(),
    X2 = vk::SampleCountFlags::TYPE_2.as_raw(),
    X4 = vk::SampleCountFlags::TYPE_4.as_raw(),
    X8 = vk::SampleCountFlags::TYPE_8.as_raw(),
    X16 = vk::SampleCountFlags::TYPE_16.as_raw(),
    X32 = vk::SampleCountFlags::TYPE_32.as_raw(),
    X64 = vk::SampleCountFlags::TYPE_64.as_raw(),
}

impl From<MSAA> for vk::SampleCountFlags {
    
    fn from(value: MSAA) -> Self {
        Self::from_raw(value.as_raw())
    }
}
