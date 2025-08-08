use super::*;

pub(crate) enum ImageSource<'a> {
    Image(Arc<Image>),
    Subresource(&'a ImageSubresourceRange),
}

impl<'a> ImageSource<'a> {

    #[inline(always)]
    pub fn _state(&self) -> ImageState {
        match self {
            Self::Image(s) => {
                s.state()
            },
            Self::Subresource(s) => {
                s.state
            },
        }
    }

    #[inline(always)]
    pub fn _layout(&self) -> vk::ImageLayout {
        match self {
            Self::Image(s) => {
                s.layout()
            },
            Self::Subresource(s) => {
                s.layout()
            }
        }
    }

    #[inline(always)]
    pub fn properties(&self) -> ImageProperties {
        match self {
            Self::Image(s) => {
                s.properties()
            },
            Self::Subresource(s) => {
                s.properties()
            },
        }
    }

    #[inline(always)]
    pub fn vk_format(&self) -> vk::Format {
        match self {
            Self::Image(s) => {
                s.vk_format()
            },
            Self::Subresource(s) => {
                s.vk_format()
            }
        }
    }

    #[inline(always)]
    pub fn samples(&self) -> MSAA {
        match self {
            Self::Image(s) => {
                s.samples()
            },
            Self::Subresource(s) => {
                s.samples()
            }
        }
    }
}

pub(crate) enum ImageSourceMut<'a> {
    Image(Arc<Image>),
    Subresource(&'a mut ImageSubresourceRange),
}

impl<'a> ImageSourceMut<'a> {

    #[inline(always)]
    pub fn state(&self) -> ImageState {
        match self {
            Self::Image(s) => {
                s.state()
            },
            Self::Subresource(s) => {
                s.state
            },
        }
    }

    #[inline(always)]
    pub fn layout(&self) -> vk::ImageLayout {
        match self {
            Self::Image(s) => {
                s.layout()
            },
            Self::Subresource(s) => {
                s.layout()
            }
        }
    }

    #[inline(always)]
    pub fn get_view(&mut self) -> Result<vk::ImageView, Error> {
        match self {
            Self::Image(s) => {
                s.get_view()
            },
            Self::Subresource(s) => {
                s.get_view()
            },
        }
    }

    #[inline(always)]
    pub fn cmd_memory_barrier(&mut self, state: ImageState, command_buffer: vk::CommandBuffer) {
        match self {
            Self::Image(s) => {
                s.cmd_memory_barrier(state, command_buffer);
            },
            Self::Subresource(s) => {
                s.cmd_memory_barrier(state, command_buffer);
            },
        }
    }
}
