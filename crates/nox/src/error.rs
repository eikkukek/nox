use crate::renderer;

#[derive(Clone, Debug)]
pub enum Error {
    RendererError(renderer::Error),
    UserError(String),
}

impl From<renderer::Error> for Error {

    fn from(value: renderer::Error) -> Self {
        Self::RendererError(value)
    }
}

impl From<String> for Error {

    fn from(value: String) -> Self {
        Self::UserError(value)
    }
}
