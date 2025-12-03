use crate::log::{self, error};

use super::*;

use core::error::Error;

#[derive(Debug)]
struct AnyError<E: Error> {
    pub err: E,
}

impl<E: Error> core::fmt::Display for AnyError<E> {

    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "some error")
    }
}

impl<E: Error + 'static> Error for AnyError<E> {

    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.err)
    }
}

pub fn fn_expand_error(target: &str, msg: &str, err: impl core::error::Error + 'static) -> Result<bool, log::LogError> {
    if let Some(&error_cause_fmt) = ERROR_CAUSE_FMT.get() {
        let err = AnyError { err, };
        if error!("{}", msg) {
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
