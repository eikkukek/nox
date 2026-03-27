use raw_window_handle::HasDisplayHandle;
use winit::event_loop::EventLoop;

use super::*;

pub struct Platform {
    pub(super) event_loop: EventLoop<RunEvent>
}

impl Default for Platform {

    fn default() -> Self {
        Self::new()
    }
}

impl Platform {

    #[inline(always)]
    pub fn new() -> Self {
        crate::log::init();
        Self {
            event_loop: EventLoop
                ::with_user_event()
                .build().expect("failed to create event loop")
        }
    }

}

impl HasDisplayHandle for Platform {

    fn display_handle(
        &self
    ) -> std::result::Result<raw_window_handle::DisplayHandle<'_>, raw_window_handle::HandleError>
    {
        self.event_loop.display_handle()
    }
}
