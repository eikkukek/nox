use super::image_state::ImageState;
use ash::vk;

pub struct DrawData {
    pub image: vk::Image,
    pub image_view: vk::ImageView,
    pub image_index: u32,
    pub image_state: ImageState,
    pub suboptimal: bool,
}

impl DrawData {

    pub fn new(
        image: vk::Image,
        image_view: vk::ImageView,
        image_index: u32,
        image_state: ImageState,
        suboptimal: bool,
    ) -> Self {
        Self {
            image,
            image_view,
            image_index,
            image_state,
            suboptimal,
        }
    }
}
