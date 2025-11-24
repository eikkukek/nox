use core::ptr::NonNull;

use nox_geom::*;

use crate::{
    collapsing_header::*,
    *
};

pub trait UiSurface {

    fn set_resizeable(&mut self, value: bool);

    fn set_clamp_width(&mut self, value: bool);

    fn set_clamp_height(&mut self, value: bool);

    fn moving(&self) -> bool;

    fn set_widget_rect_max(&mut self, max: Vec2);

    fn painter_storage(&self) -> &mut PainterStorage;

    fn activate_collapsing_header(
        &mut self,
        label: &str,
    ) -> (&mut CollapsingHeader, CollapsingHeaderId);

    fn get_collapsing_header(&self, id: CollapsingHeaderId) -> Option<&CollapsingHeader>;

    fn get_collapsing_header_mut(&mut self, id: CollapsingHeaderId) -> Option<&mut CollapsingHeader>;
    
    fn add_text(
        &mut self,
        text: Text
    ) -> usize;

    fn get_text_mut(&mut self, index: usize) -> Option<&mut Text>;

    fn reaction_text(
        &mut self,
        style: &impl UiStyle,
        text_renderer: &mut TextRenderer,
        id: ReactionId,
        text: &str,
    ) -> &mut Text;

    fn animated_bool(&mut self, id: ReactionId, value: bool) -> f32;

    unsafe fn tmp_data<T>(&self, count: usize) -> Option<NonNull<T>>;
}

pub trait UiReactSurface {

    type Surface: UiSurface;

    fn ui_surface(&self) -> &Self::Surface;

    fn ui_surface_mut(&mut self) -> &mut Self::Surface;

    fn reaction_from_ref<T: ?Sized>(
        &self,
        value: &T,
        f: impl FnMut(&mut Self::Surface, &T, &mut ReactionEntry),
    ) -> &mut Reaction;

    fn reaction_from_mut<T: ?Sized>(
        &mut self,
        value: &mut T,
        f: impl FnMut(&mut Self::Surface, &mut T, &mut ReactionEntry),
    ) -> &mut Reaction;

    fn get_reaction(&self, id: ReactionId) -> Option<&ReactionEntry>;

    fn get_reaction_mut(&mut self, id: ReactionId) -> Option<&mut ReactionEntry>;
}
