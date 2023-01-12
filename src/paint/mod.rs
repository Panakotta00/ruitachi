mod painter;

use skia_safe::{Font, Paint};
pub use painter::Painter;

pub struct TextStyle {
    pub font: Font,
    pub color: Paint,
}

impl Default for TextStyle {
    fn default() -> Self {
        Self{
            font: Font::default(),
            color: Paint::default(),
        }
    }
}