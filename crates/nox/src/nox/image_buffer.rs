use std::{ffi::CString};

use core::{
    ptr::NonNull,
    slice,
    ops::Deref,
};

use stb_image::stb_image;

use crate::renderer::image::Dimensions;

pub struct ImageBuffer {
    pub(crate) buffer: NonNull<u8>,
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) channels: u32,
}

impl ImageBuffer {

    pub fn new(filename: &str, channels: u32) -> Result<Self, String> {
        if channels > 4 {
            return Err(format!("channels must be less or equal to 4 ( requested channels: {} )", channels))
        }
        let mut x = 0;
        let mut y = 0;
        let mut ch = 0;
        let filename = CString
            ::new(filename)
            .unwrap();
        let img = unsafe {
            stb_image::stbi_load(filename.as_ptr(), &mut x, &mut y, &mut ch, channels as i32)
        };
        if img.is_null() {
            unsafe {
                let err = std::ffi::CString::from_raw(stb_image::stbi_failure_reason() as *mut i8);
                return Err(
                    err
                        .to_str()
                        .unwrap_or("<failed to convert C string>")
                        .into()
                )
            }
        }
        Ok(Self {
            buffer: NonNull::new(img).unwrap(),
            width: x as u32,
            height: y as u32,
            channels: if channels == 0 { ch as u32 } else { channels },
        })
    }

    pub fn as_slice(&self) -> &[u8] {
        unsafe {
            slice::from_raw_parts(
                self.buffer.as_ptr(),
                self.width as usize * self.height as usize,
            )
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn dimensions(&self) -> Dimensions {
        Dimensions::new(self.width, self.height, 1)
    }
    
    pub fn channels(&self) -> u32 {
        self.channels
    }
}

impl Deref for ImageBuffer {

    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl Drop for ImageBuffer {

    fn drop(&mut self) {
        unsafe {
            stb_image::stbi_image_free(self.buffer.as_ptr() as _);
        }
    }
}

unsafe impl Send for ImageBuffer {}
unsafe impl Sync for ImageBuffer {}
