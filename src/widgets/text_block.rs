use crate::paint::{Painter, TextStyle};
use crate::util::{Geometry, WidgetRef};
use crate::widgets::{Widget, WidgetState};
use cgmath::Vector2;
use skia_safe::{scalar, Point};

pub struct TextBlockWidget {
	widget: WidgetState,
	text: String,
	text_style: TextStyle,
}

pub struct TextBlockWidgetBuilder(TextBlockWidget);

/// A Widget showing some simple text.
///
/// The desired size is the size the text would consume if the text were in one line.
impl TextBlockWidget {
	pub fn new() -> TextBlockWidgetBuilder {
		TextBlockWidgetBuilder(TextBlockWidget {
			widget: WidgetState::default(),
			text: String::default(),
			text_style: TextStyle::default(),
		})
	}
}

impl TextBlockWidgetBuilder {
	pub fn text(mut self, text: String) -> Self {
		self.0.text = text;
		self
	}

	pub fn build(mut self) -> WidgetRef<TextBlockWidget> {
		WidgetRef::new(self.0)
	}
}

impl Widget for TextBlockWidget {
	fn widget_state(&self) -> &WidgetState {
		&self.widget
	}

	fn widget_state_mut(&mut self) -> &mut WidgetState {
		&mut self.widget
	}

	fn paint(&self, geometry: Geometry, layer: i32, painter: &mut Painter) -> i32 {
		painter.draw_str(
			&self.text,
			Point::new(0.0, 0.0),
			&self.text_style.font,
			&self.text_style.color,
		);
		layer
	}

	fn get_desired_size(&self) -> Vector2<scalar> {
		let (_, rect) = self
			.text_style
			.font
			.measure_str(&self.text, Some(&self.text_style.color));
		Vector2::new(rect.height(), rect.width())
	}
}
