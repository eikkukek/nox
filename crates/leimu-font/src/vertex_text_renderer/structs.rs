use ahash::AHashMap;
use compact_str::CompactString;

use nox_geom::Vec2;

use super::*;

#[derive(Clone, Copy)]
pub struct TextSegmentRef<'a, H> {
    pub text: &'a str,
    pub font: &'a H,
}

pub fn text_segment<'a, H>(text: &'a str, font: &'a H) -> TextSegmentRef<'a, H> {
    TextSegmentRef { text, font }
}

#[derive(Clone, PartialEq, Eq)]
pub struct TextSegmentOwned<H> {
    pub text: CompactString,
    pub font: H,
}

pub fn text_segment_owned<H>(text: &str, font: H) -> TextSegmentOwned<H> {
    TextSegmentOwned { text: text.into(), font }
}

pub trait TextSegment<H> {

    fn text(&self) -> &str;

    fn font(&self) -> &H;
}

impl<'a, H> TextSegment<H> for TextSegmentRef<'a, H> {

    fn text(&self) -> &str {
        self.text
    }

    fn font(&self) -> &H {
        self.font
    }
}

impl<H> TextSegment<H> for TextSegmentOwned<H> {

    fn text(&self) -> &str {
        &self.text
    }

    fn font(&self) -> &H {
        &self.font
    }
}

#[derive(Clone)]
pub struct InstancedText {
    pub trigs: Arc<GlyphTriangles>,
    pub offsets: Vec<VertexOffset>,
}

#[derive(Default, Clone)]
pub struct RenderedText {
    pub text: Vec<(char, InstancedText)>,
    pub text_width: f32,
    pub row_height: f32,
    pub text_rows: u32,
    pub last_row_width: f32,
}

impl RenderedText {

    pub fn get_offset_mut(&mut self, text_offset: TextOffset) -> Option<&mut VertexOffset> {
        for (c, text) in &mut self.text {
            if *c == text_offset.char {
                return text.offsets.get_mut(text_offset.offset_index? as usize)
            }
        }
        None
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.text.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }

    #[inline]
    pub fn iter(&self) -> slice::Iter<'_, (char, InstancedText)> {
        self.text.iter()
    }
}

#[derive(Default)]
pub struct CombinedRenderedText<UserInstanceData> {
    pub text: AHashMap<char, (InstancedText, Vec<UserInstanceData>)>,
}

impl<UserInstanceData> CombinedRenderedText<UserInstanceData> {

    #[inline(always)]
    pub fn new() -> Self {
        Self {
            text: AHashMap::default(),
        }
    }

    pub fn add_text(
        &mut self,
        text: &RenderedText,
        offset: impl Into<Vec2>,
        user_data: UserInstanceData,
    )
        where 
            UserInstanceData: Copy
    {
        let offset = offset.into();
        for (c, t) in text.iter() {
            let (instanced, data) = self.text
                .entry(*c)
                .or_insert_with(|| (InstancedText {
                        trigs: t.trigs.clone(), offsets: Default::default(),
                    }, Default::default(),
                ));
            for off in &t.offsets {
                instanced.offsets.push(VertexOffset { offset: (offset + off.offset.into()).into() });
                if size_of::<UserInstanceData>() != 0 {
                    data.push(user_data);
                }
            }
        }
    }

    #[inline(always)]
    pub fn iter(&self) -> impl Iterator<Item = (char, &InstancedText, &[UserInstanceData])> {
        self.text.iter().map(|(c, (t, u))| (*c, t, u.as_ref()))
    }

    #[inline(always)]
    pub fn clear(&mut self) {
        self.text.clear();
    }
}

pub(super) struct FaceCache<F>
{
    pub face: F,
    pub trigs: AHashMap<char, Option<Arc<GlyphTriangles>>>,
    pub offsets: AHashMap<char, Option<Vec<VertexOffset>>>,
}
