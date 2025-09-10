use core::hash::Hash;

use nox_font::{VertexTextRenderer, Face};

pub struct Workspace<'a, FontHash>
    where
        FontHash: Clone + PartialEq + Eq + Hash
{
    text_renderer: VertexTextRenderer<'a, FontHash>,
}

impl<'a, FontHash> Workspace<'a, FontHash>
    where
        FontHash: Clone + PartialEq + Eq + Hash
{

    pub fn new(
        fonts: impl IntoIterator<Item = (FontHash, Face<'a>)>,
        font_curve_depth: u32,
    ) -> Self
    {
        Self {
            text_renderer: VertexTextRenderer::new(fonts, font_curve_depth),
        }
    }
}
