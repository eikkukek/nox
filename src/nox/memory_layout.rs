#[derive(Clone, Copy)]
pub struct MemoryLayout {
    temp_size: usize,
}

impl MemoryLayout {

    pub fn default() -> Self {
        Self {
            temp_size: 1 << 18,
        }
    }

    pub fn temp_size(&self) -> usize {
        self.temp_size
    }
}
