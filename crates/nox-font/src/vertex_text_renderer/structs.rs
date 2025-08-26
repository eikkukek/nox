use super::*;

pub struct TextSegment<'a, H> {
    pub text: &'a str,
    pub font: H,
}

pub fn text_segment<'a, H>(text: &'a str, font: H) -> TextSegment<'a, H> {
    TextSegment { text, font }
}

#[derive(Clone)]
pub struct InstancedText {
    pub trigs: Arc<GlyphTriangles>,
    pub offsets: GlobalVec<VertexOffset>,
}

#[derive(Default, Clone)]
pub struct RenderedText {
    pub text: GlobalVec<InstancedText>,
    pub text_width: f32,
    pub font_height: f32,
    pub text_rows: u32,
}

pub(super) struct FaceCache<'a> {
    pub face: Face<'a>,
    pub trigs: FxHashMap<char, Option<Arc<GlyphTriangles>>>,
    pub offsets: FxHashMap<char, Option<GlobalVec<VertexOffset>>>,
}
