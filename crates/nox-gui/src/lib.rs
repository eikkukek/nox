#[macro_use]

mod ring_buffer;
mod shaders;
mod color;
mod style;
mod helpers;
mod text;
mod vertex_range;
mod widget;
mod slider;
mod drag_value;
mod color_picker;
mod input_text;
mod selectable_tag;
mod combo_box;
mod hover_window;
mod drop_down;
mod scroll_bar;
pub mod image;
pub mod surface;
pub mod collapsing_header;
mod ui_ctx;
mod on_top_contents;
mod painter;
mod window;
mod reaction;
mod workspace;

pub use ring_buffer::*;
use shaders::*;

pub use color::*;
pub use style::*;
pub use text::*;
pub use vertex_range::VertexRange;
pub use widget::*;
pub use slider::*;
pub use drag_value::*;
pub use scroll_bar::*;
pub use hover_window::*;
pub use drop_down::*;
pub use ui_ctx::*;
pub use window::*;
pub use workspace::*;
pub use on_top_contents::*;
pub use painter::*;

pub use nox_font as font;
pub use nox_geom as geom;

pub use compact_str::CompactString;

pub use input_text::{InputText, InputTextData};
pub use color_picker::ColorPicker;
pub use selectable_tag::SelectableTag;
pub use combo_box::{ComboBox, ComboBoxBuilder};
pub use image::{ImageSource, ImageLoader};
pub use reaction::*;
pub use helpers::*;

pub type TextRenderer<'a> = font::VertexTextRenderer<'a, compact_str::CompactString>;
