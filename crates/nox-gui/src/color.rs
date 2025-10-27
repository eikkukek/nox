use core::f32::consts;

use core::fmt::Display;

use nox::VkFormat;

pub trait Color: Copy + Display {

    fn from_srgba(value: ColorSRGBA) -> Self;

    fn to_srgba(self) -> ColorSRGBA;

    fn from_hsva(value: ColorHSVA) -> Self;

    fn to_hsva(self) -> ColorHSVA;
}

#[derive(Default, Clone, Copy, PartialEq, Debug)]
pub struct ColorSRGBA {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub alpha: f32,
}

impl ColorSRGBA {

    pub const VK_FORMAT: VkFormat = VkFormat::R32G32B32A32_SFLOAT;

    #[inline(always)]
    pub const fn new(r: f32, g: f32, b: f32, alpha: f32) -> Self {
        Self {
            r,
            g,
            b,
            alpha,
        }
    }

    #[inline(always)]
    pub const fn red(alpha: f32) -> Self {
        Self {
            r: 1.0,
            g: 0.0,
            b: 0.0,
            alpha,
        }
    }

    #[inline(always)]
    pub const fn white(alpha: f32) -> Self {
        Self {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            alpha,
        }
    }

    #[inline(always)]
    pub const fn black(alpha: f32) -> Self {
        Self {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            alpha,
        }
    }

    #[inline(always)]
    pub const fn with_alpha(mut self, alpha: f32) -> Self {
        self.alpha = alpha;
        self
    }

    #[inline(always)]
    pub const fn scale_alpha(mut self, scale: f32) -> Self {
        self.alpha *= scale;
        self
    }
}

impl Color for ColorSRGBA {

    #[inline(always)]
    fn from_srgba(value: ColorSRGBA) -> Self {
        value
    }

    #[inline(always)]
    fn to_srgba(self) -> ColorSRGBA {
        self
    }

    #[inline(always)]
    fn from_hsva(value: ColorHSVA) -> Self {
        let map = |n: f32| -> f32 {
            let k = (n + value.hue / consts::FRAC_PI_3) % 6.0;
            let ch = value.val - value.val * value.sat * k.min(4.0 - k).min(1.0).max(0.0);
            if ch <= 0.04045 {
                ch / 12.92
            } else {
                ((ch + 0.055) / 1.055).powf(2.4)
            }
        };
        Self {
            r: map(5.0),
            g: map(3.0),
            b: map(1.0),
            alpha: value.alpha,
        }
    }

    #[inline(always)]
    fn to_hsva(self) -> ColorHSVA {
        let map = |ch: f32| -> f32 {
            if ch <= 0.0031308 {
                12.92 * ch
            } else {
                1.055 * ch.powf(1.0 / 2.4) - 0.055
            }
        };
        let r = map(self.r);
        let g = map(self.g);
        let b = map(self.b);
        let alpha = self.alpha;
        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let chroma = max - min;
        let mut hue = match (chroma, max) {
            (c, _) if c.abs() < f32::EPSILON => 0.0,
            (c, max) if max == r => ((g - b) / c) % 6.0,
            (c, max) if max == g => (b - r) / c + 2.0,
            (c, max) if max == b => (r - g) / c + 4.0,
            _ => 0.0,
        } * consts::FRAC_PI_3;
        if hue < 0.0 {
            hue += consts::TAU;
        }
        //let val = max;
        let val = max;
        let sat =
            if max.abs() < f32::EPSILON {
                0.0
            } else {
                chroma / val
            };
        ColorHSVA {
            hue,
            sat,
            val,
            alpha,
        }
    }
}

impl Display for ColorSRGBA {

    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "(r: {:.2}, g: {:.2}, b: {:.2}, a: {:.2})",
            self.r,
            self.g,
            self.b,
            self.alpha,
        )
    }
}

#[derive(Default, Clone, Copy, PartialEq, Debug)]
pub struct ColorHSVA {
    pub hue: f32,
    pub sat: f32,
    pub val: f32,
    pub alpha: f32,
}

impl ColorHSVA {

    #[inline(always)]
    pub const fn new(hue: f32, sat: f32, val: f32, alpha: f32) -> Self {
        Self {
            hue,
            sat,
            val,
            alpha,
        }
    }

    #[inline(always)]
    pub const fn white(alpha: f32) -> Self {
        Self {
            hue: 0.0,
            sat: 0.0,
            val: 1.0,
            alpha,
        }
    }

    #[inline(always)]
    pub const fn black(alpha: f32) -> Self {
        Self {
            hue: 0.0,
            sat: 0.0,
            val: 0.0,
            alpha,
        }
    }
}

impl Color for ColorHSVA {

    fn from_srgba(value: ColorSRGBA) -> Self {
        let map = |ch: f32| -> f32 {
            if ch <= 0.0031308 {
                12.92 * ch
            } else {
                1.055 * ch.powf(1.0 / 2.4) - 0.055
            }
        };
        let r = map(value.r);
        let g = map(value.g);
        let b = map(value.b);
        let alpha = value.alpha;
        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let chroma = max - min;
        let mut hue = match (chroma, max) {
            (c, _) if c.abs() < f32::EPSILON => 0.0,
            (c, max) if max == r => ((g - b) / c) % 6.0,
            (c, max) if max == g => (b - r) / c + 2.0,
            (c, max) if max == g => (r - g) / c + 4.0,
            _ => 0.0,
        } * consts::FRAC_PI_3;
        if hue < 0.0 {
            hue += consts::TAU;
        }
        let val = max;
        let sat =
            if max.abs() < f32::EPSILON {
                0.0
            } else {
                chroma / val
            };
        Self {
            hue,
            sat,
            val,
            alpha,
        }
    }

    fn to_srgba(self) -> ColorSRGBA {
        let map = |n: f32| -> f32 {
            let k = (n + self.hue / consts::FRAC_PI_3) % 6.0;
            let ch = self.val - self.val * self.sat * k.min(4.0 - k).min(1.0).max(0.0);
            if ch <= 0.04045 {
                ch / 12.92
            } else {
                ((ch + 0.055) / 1.055).powf(2.4)
            }
        };
        ColorSRGBA {
            r: map(5.0),
            g: map(3.0),
            b: map(1.0),
            alpha: self.alpha,
        }
    }

    fn from_hsva(value: ColorHSVA) -> Self {
        value
    }

    fn to_hsva(self) -> ColorHSVA {
        self
    }
}

impl Display for ColorHSVA {

    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "(h: {:.2}, s: {:.2}, v: {:.2}, a: {:.2})",
            self.hue,
            self.sat,
            self.val,
            self.alpha,
        )
    }
}

#[derive(Default, Clone, Copy, Debug)]
pub struct ColorRGBA {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub alpha: u8,
}

impl ColorRGBA {

    #[inline(always)]
    pub fn map_channel(srgb_ch: f32) -> u8 {
        (if srgb_ch <= 0.0031308 {
            12.92 * srgb_ch
        } else {
            1.055 * srgb_ch.powf(1.0 / 2.4) - 0.055
        } * 255.0).clamp(0.0, 255.0).round() as u8
    }

    #[inline(always)]
    pub fn from_srgba(value: ColorSRGBA) -> Self {
        let map = |ch: f32| -> u8 {
            (if ch <= 0.0031308 {
                12.92 * ch
            } else {
                1.055 * ch.powf(1.0 / 2.4) - 0.055
            } * 255.0).clamp(0.0, 255.0).round() as u8
        };
        Self {
            r: map(value.r),
            g: map(value.g),
            b: map(value.b),
            alpha: (value.alpha * 255.0) as u8,
        }
    }

    #[inline(always)]
    pub fn to_srgba(self) -> ColorSRGBA {
        let map = |ch: u8| -> f32 {
            let ch = ch as f32 / 255.0;
            if ch <= 0.04045 {
                ch / 12.92
            } else {
                ((ch + 0.055) / 1.055).powf(2.4)
            }
        };
        ColorSRGBA {
            r: map(self.r),
            g: map(self.g),
            b: map(self.b),
            alpha: self.alpha as f32 / 255.0,
        }
    }
}

    /*
    #[inline(always)]
    fn from_oklab(value: ColorOklab) -> Self {

        let l = value.lightness + 0.3963377774 * value.a + 0.2158037573 * value.b;
        let m = value.lightness - 0.1055613458 * value.a - 0.0638541728 * value.b;
        let s = value.lightness - 0.0894841775 * value.a - 1.2914855480 * value.b;

        let l_ = l * l * l;
        let m_ = m * m * m;
        let s_ = s * s * s;

        let r = 4.0767416621 * l_ - 3.3077115913 * m_ + 0.2309699292 * s_;
        let g = -1.2684380046 * l_ + 2.6097574011 * m_ - 0.3413193965 * s_;
        let b = -0.0041960863 * l_ - 0.7034186147 * m_ + 1.7076147010 * s_;

        return Self {
            r,
            g,
            b,
            a: value.alpha,
        };
    }

    #[inline(always)]
    fn to_oklab(self) -> ColorOklab {

        let r = self.r;
        let g = self.g;
        let b = self.b;

        let l = 0.4122214708 * r + 0.5363325363 * g + 0.0514489929 * b;
        let m = 0.2119034982 * r + 0.6806995451 * g + 0.1073969566 * b;
        let s = 0.0883024619 * r + 0.2817188376 * g + 0.6299787005 * b;

        let l_ = l.cbrt();
        let m_ = m.cbrt();
        let s_ = s.cbrt();

        return ColorOklab {
            lightness: 0.2104542553 * l_ + 0.7936177850 * m_ - 0.0040720468 * s_,
            a: 1.9779984951 * l_ - 2.4285922050 * m_ + 0.4505937099 * s_,
            b: 0.0259040371 * l_ + 0.7827717662 * m_ - 0.8086757660 * s_,
            alpha: self.a,
        };
    }
    */
