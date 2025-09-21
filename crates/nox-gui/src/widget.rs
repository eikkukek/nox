pub use nox_geom::{
    *,
    shapes::*,
};

pub(crate) struct Widget {
    pub main_rect: Rect,
    pub position: Vec2,
}

impl Widget {

    pub fn new(size: [f32; 2], position: [f32; 2]) -> Self {
        let half_size = vec2(size[0] * 0.5, size[1] * 0.5);
        Self {
            main_rect: rect(-half_size, half_size, 0.0),
            position: position.into(),
        }
    }
}
