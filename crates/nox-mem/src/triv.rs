mod triv;
mod maybe_triv;

pub use triv::Triv;
pub use maybe_triv::MaybeTriv;

pub fn is_triv<T>() -> bool {
    <T as MaybeTriv>::is_triv()
}
