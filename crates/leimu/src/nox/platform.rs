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
        log::init();
        log::info_fmt(|fmt| {
            fmt.text("INFO:  ", |spec| spec.with_color_spec(|spec| {
                spec.set_fg(Some(log::Color::Green)).set_bold(true);
            })).message(|spec| spec);
        });
        log::warn_fmt(|fmt| {
            fmt.text("WARN:  ", |spec| spec.with_color_spec(|spec| {
                spec.set_fg(Some(log::Color::Yellow)).set_bold(true);
            })).message(|spec| spec);
        });
        log::error_fmt(|fmt| {
            fmt.text("ERROR: ", |spec| spec.with_color_spec(|spec| {
                spec.set_fg(Some(log::Color::Red)).set_bold(true);
            })).message(|spec| spec);
        });
        log::debug_fmt(|fmt| {
            fmt.text("DEBUG: ", |spec| spec.with_color_spec(|spec| {
                spec.set_fg(Some(log::Color::Blue)).set_bold(true);
            })).message(|spec| spec);
        });
        log::trace_fmt(|fmt| {
            fmt.text("TRACE: ", |spec| spec.with_color_spec(|spec| {
                spec.set_fg(Some(log::Color::Rgb(130, 130, 130))).set_bold(true);
            })).message(|spec| spec);
        }); 
        if expand::ERROR_CAUSE_FMT.get().is_none() {
            let mut error_cause_fmt = log::LogFmt::default();
            log::LogFmtBuilder::new(&mut error_cause_fmt)
                .text("       caused by: ", |spec| spec.with_color_spec(|spec| {
                    spec.set_fg(Some(log::Color::Magenta)).set_bold(true);
                })).message(|spec| spec);
            expand::ERROR_CAUSE_FMT.set(log::custom_fmt(error_cause_fmt)).ok();
        }
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
