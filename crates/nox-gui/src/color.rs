use nox::VkFormat;

#[derive(Default, Clone, Copy, PartialEq, Debug)]
pub struct ColorRGBA {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl ColorRGBA {

    pub const VK_FORMAT: VkFormat = VkFormat::R32G32B32A32_SFLOAT;

    #[inline(always)]
    pub fn from_rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a, }
    }

    #[inline(always)]
    pub const fn white() -> Self {
        Self {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        }
    }

    #[inline(always)]
    pub const fn black() -> Self {
        Self {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        }
    }

    #[inline(always)]
    pub const fn transparent_black() -> Self {
        Self {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 0.0,
        }
    }
}
