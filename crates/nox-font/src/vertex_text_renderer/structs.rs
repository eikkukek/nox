use std::collections::hash_map;

use core::marker::PhantomData;

use rustc_hash::FxHashMap;
use compact_str::CompactString;

use nox::mem::CapacityError;
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
    pub offsets: GlobalVec<VertexOffset>,
}

#[derive(Default, Clone)]
pub struct RenderedText {
    pub text: GlobalVec<(char, InstancedText)>,
    pub text_width: f32,
    pub row_height: f32,
    pub text_rows: u32,
    pub last_row_width: f32,
}

impl RenderedText {

    pub fn get_offset_mut(&mut self, text_offset: TextOffset) -> Option<&mut VertexOffset> {
        for (c, text) in &mut self.text {
            if *c == text_offset.char {
                return Some(text.offsets.get_mut(text_offset.offset_index? as usize)?)
            }
        }
        return None
    }

    pub fn iter(&self) -> slice::Iter<'_, (char, InstancedText)> {
        self.into_iter()
    }
}

impl<'a> IntoIterator for &'a RenderedText {
    
    type Item = &'a (char, InstancedText);
    type IntoIter = slice::Iter<'a, (char, InstancedText)>;

    fn into_iter(self) -> Self::IntoIter {
        self.text.iter()
    }
}

pub struct CombinedRenderedText<UserInstanceData, V: Vector<UserInstanceData>> {
    pub text: FxHashMap<char, (InstancedText, V)>,
    _marker: PhantomData<UserInstanceData>,
}

impl<UserInstanceData, V: Default + Vector<UserInstanceData>> CombinedRenderedText<UserInstanceData, V> {

    #[inline(always)]
    pub fn new() -> Self {
        Self {
            text: FxHashMap::default(),
            _marker: PhantomData,
        }
    }

    pub fn add_text(
        &mut self,
        text: &RenderedText,
        offset: impl Into<Vec2>,
        user_data: UserInstanceData,
    ) -> Result<(), CapacityError>
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
                    data.push(user_data)?;
                }
            }
        }
        Ok(())
    }

    #[inline(always)]
    pub fn iter<'a>(&'a self) -> hash_map::Iter<'a, char, (InstancedText, V)> {
        self.text.iter()
    }

    #[inline(always)]
    pub fn clear(&mut self) {
        self.text.clear();
    }
}

impl<'a, UserInstanceData, V: Vector<UserInstanceData>> IntoIterator for &'a CombinedRenderedText<UserInstanceData, V> {
    
    type Item = (&'a char, &'a (InstancedText, V));
    type IntoIter = hash_map::Iter<'a, char, (InstancedText, V)>;

    fn into_iter(self) -> Self::IntoIter {
        self.text.iter()
    }
}

impl<UserInstanceData, V: Vector<UserInstanceData>> Default for CombinedRenderedText<UserInstanceData, V> {

    #[inline(always)]
    fn default() -> Self {
        Self {
            text: FxHashMap::default(),
            _marker: PhantomData,
        }
    }
}

pub(super) struct FaceCache<'a> {
    pub face: Face<'a>,
    pub trigs: FxHashMap<char, Option<Arc<GlyphTriangles>>>,
    pub offsets: FxHashMap<char, Option<GlobalVec<VertexOffset>>>,
}
