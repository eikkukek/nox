use ash::vk;

use nox_mem::AsRaw;

#[repr(i32)]
#[derive(Default, Clone, Copy, AsRaw, Debug)]
pub enum IndexType {
    U16 = vk::IndexType::UINT16.as_raw(),
    #[default]
    U32 = vk::IndexType::UINT32.as_raw(),
}

impl IndexType {
    
    pub fn index_size(self) -> u64 {
        match self {
            Self::U16 => 2,
            Self::U32 => 4,
        }
    }
}

impl From<IndexType> for vk::IndexType {

    fn from(value: IndexType) -> Self {
        vk::IndexType::from_raw(value.as_raw())
    }
}
