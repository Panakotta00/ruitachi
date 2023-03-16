use crate::{
	paint::{Painter, TextStyle},
	util::{Geometry, WidgetRef},
	widgets::{Widget, WidgetState},
};
use cgmath::Vector2;
use skia_safe::{scalar, Point};
use crate::widgets::{Arrangements, Children};
use crate::widgets::leaf_widget::{LeafState, LeafWidget};

pub struct TextBlockWidget {
	leaf: LeafState,
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
			leaf: Default::default(),
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

	pub fn build(self) -> WidgetRef<TextBlockWidget> {
		WidgetRef::new(self.0)
	}
}

impl Widget for TextBlockWidget {
	fn widget_state(&self) -> &WidgetState {
		&self.leaf.widget
	}

	fn widget_state_mut(&mut self) -> &mut WidgetState {
		&mut self.leaf.widget
	}

	fn paint(&self, _geometry: Geometry, layer: i32, painter: &mut Painter) -> i32 {
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

	fn get_children(&self) -> Children {
		self.leaf_get_children()
	}

	fn arrange_children(&mut self, geometry: Geometry) {
		self.leaf_arrange_children(geometry)
	}

	fn get_arranged_children(&self) -> Arrangements {
		self.leaf_get_arranged_children()
	}

	fn cached_geometry(&self) -> Geometry {
		self.leaf_cached_geometry()
	}
}

impl LeafWidget for TextBlockWidget {
	fn leaf_state(&self) -> &LeafState {
		&self.leaf
	}

	fn leaf_state_mut(&mut self) -> &mut LeafState {
		&mut self.leaf
	}
}
