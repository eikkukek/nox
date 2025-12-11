#[macro_use]

mod log;
mod error;
pub mod fmt;

pub use error::LogError;
pub use fmt::{LogFmt, LogFmtBuilder};

pub type Result<T> = core::result::Result<T, LogError>;

pub use log::*;
