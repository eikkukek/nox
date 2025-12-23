pub struct Ref<'a, T> {
    pub(super) initialized: &'a bool,
    pub(super) t: *mut T,
}

impl<'a, T> Ref<'a, T> {

    pub fn get(&self) -> &T {
        if !self.initialized {
            panic!("value not initialized, OnInit needs to be passed to Nox before using Ref")
        }
        unsafe {
            &*self.t
        }
    }

    pub fn get_mut(&mut self) -> &mut T {
        if !self.initialized {
            panic!("value not initialized, OnInit needs to be passed to Nox before using Ref")
        }
        unsafe {
            &mut *self.t
        }
    }
}
