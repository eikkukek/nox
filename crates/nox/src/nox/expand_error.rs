use core::error;

use nox_error::Error;

use super::ERROR_CAUSE_FMT;

use crate::log::{self, Result, error, warn};

pub fn fn_expand_error(target: &str, err: Error) -> Result<bool> {
    if let Some(&error_cause_fmt) = ERROR_CAUSE_FMT.get() {
        if error!("{}", err) {
            let mut err: &dyn error::Error = &err;
            while let Some(source) = err.source() {
                err = source;
                log::log(
                    target,
                    log::LevelFmt::Other(error_cause_fmt, log::Level::Error),
                    format_args!("{}", err)
                )?;
            }
            return Ok(true)
        }
    }
    Ok(false)
}

pub fn fn_expand_warn(target: &str, err: Error) -> Result<bool> {
        if let Some(&error_cause_fmt) = ERROR_CAUSE_FMT.get() {
        if warn!("{}", err) {
            let mut err: &dyn error::Error = &err;
            while let Some(source) = err.source() {
                err = source;
                log::log(
                    target,
                    log::LevelFmt::Other(error_cause_fmt, log::Level::Warn),
                    format_args!("{}", err)
                )?;
            }
            return Ok(true)
        }
    }
    Ok(false)
}

#[macro_export]
macro_rules! expand_error {
    ($err:expr) => {
        $crate::error_util::fn_expand_error(module_path!(), $err)
            .unwrap_or(false)
    };
}

#[macro_export]
macro_rules! expand_warn {
    ($err:expr) => {
        $crate::error_util::fn_expand_warn(module_path!(), $err)
            .unwrap_or(false)
    };
}
