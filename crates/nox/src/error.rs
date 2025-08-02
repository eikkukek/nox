use crate::renderer;

#[derive(Clone, Debug)]
pub enum Error {
    RendererError(renderer::Error),
}

impl From<renderer::Error> for Error {

    fn from(value: renderer::Error) -> Self {
        Self::RendererError(value)
    }
}
