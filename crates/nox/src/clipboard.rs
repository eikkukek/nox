use raw_window_handle::{HasDisplayHandle, RawDisplayHandle};

use winit::window::Window;

pub enum Clipboard {
    None,
    Wayland(smithay_clipboard::Clipboard),
    X11,
    Windows,
}

impl Clipboard {

    pub fn new(
        window: &Window,
    ) -> Result<Self, raw_window_handle::HandleError> {
        match window.display_handle()?.as_raw() {
            RawDisplayHandle::Wayland(display) => {
                Ok(Clipboard::Wayland(unsafe { smithay_clipboard::Clipboard::new(
                    display.display.as_ptr()
                )}))
            },
            RawDisplayHandle::Xlib(_) => {
                Ok(Self::X11)
            }
            RawDisplayHandle::Windows(_) => {
                Ok(Self::Windows)
            },
            _ => Ok(Self::None), 
        }
    }

    pub fn get(&self) -> Option<String> {
        match self {
            Self::None => {
                None
            },
            Self::Wayland(cb) => {
                if let Ok(contents) = cb.load() {
                    Some(contents)
                } else {
                    None
                }
            },
            Self::X11 => {
                None
            },
            Self::Windows => {
                None
            }
        }
    }

    pub fn set(&self, contents: &str) {
        match self {
            Self::None => {},
            Self::Wayland(cb) => {
                cb.store(contents);
            },
            Self::X11 => {},
            Self::Windows => {},
        }
    }
}
