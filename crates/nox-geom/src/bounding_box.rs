use crate::Vec2;

#[derive(Default, Clone, Copy, PartialEq, Debug)]
pub struct BoundingRect {
    pub min: Vec2,
    pub max: Vec2,
}

impl BoundingRect {

    #[inline(always)]
    pub fn from_min_max(min: Vec2, max: Vec2) -> Self {
        Self {
            min,
            max,
        }
    }

    #[inline(always)]
    pub fn from_center_size(center: Vec2, size: Vec2) -> Self {
        let half_size = size * 0.5;
        Self {
            min: center - half_size,
            max: center + half_size,
        }
    }

    #[inline(always)]
    pub fn point_inside(
        &self,
        point: Vec2,
    ) -> bool
    {
        self.min.x < point.x && self.max.x > point.x &&
        self.min.y < point.y && self.max.y > point.y
    }
}
