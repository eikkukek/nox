use crate::{
    error::SomeError,
    log::{self, error},
};

use super::*;

pub fn fn_expand_error(target: &str, msg: &str, err: impl core::error::Error + 'static) -> Result<bool, log::LogError> {
    if let Some(&error_cause_fmt) = ERROR_CAUSE_FMT.get() {
        let err = SomeError::new(msg, err);
        if error!("{}", err) {
            let mut source = err.source();
            while let Some(err) = source {
                log::log(
                    target,
                    log::LevelFmt::Other(error_cause_fmt, log::Level::Error),
                    format_args!("{}", err)
                )?;
                source = err.source();
            }
            return Ok(true)
        }
    }
    Ok(false)
}

#[macro_export]
macro_rules! expand_error {
    ($msg:expr, $err:expr) => {
        $crate::fn_expand_error(module_path!(), $msg, $err)
            .unwrap_or(false)
    };
}
