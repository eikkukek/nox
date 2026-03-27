use crate::gpu;

pub(super) enum RunEvent {
    Tick,
}

pub enum Event {
    /// Nox has been initialized.
    ///
    /// Gets called once at the beginning before any other events.
    Initialized,
    /// Nox is updating.
    ///
    /// Happens once per frame before any GPU work.
    Update,
    /// A [`gpu::Gpu`] event has happened.
    GpuEvent(gpu::Event),
    /// Nox is cleaning up.
    ///
    /// This is the last event that gets called.
    CleanUp,
}
