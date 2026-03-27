use super::prelude::*;

pub enum Event {
    /// The swapchain for surface with `surface_id` has been (re)created.
    SwapchainCreated {
        surface_id: SurfaceId,
        new_format: Format,
        new_size: (u32, u32),
    },
}
