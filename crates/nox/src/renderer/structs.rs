use ash::vk;

#[derive(Default, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Offset2D {
    pub x: i32,
    pub y: i32
}

impl From<Offset2D> for vk::Offset2D {

    fn from(value: Offset2D) -> Self {
        Self {
            x: value.x,
            y: value.y,
        }
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Offset3D {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl Offset3D {

    pub fn new(x: i32, y: i32, z: i32) -> Self
    {
        Self {x, y, z}
    }
}

impl From<Offset3D> for vk::Offset3D {

    fn from(value: Offset3D) -> Self {
        Self {
            x: value.x,
            y: value.y,
            z: value.z,
        }
    }
}
