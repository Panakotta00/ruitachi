use std::cell::{Ref, RefMut};
use crate::{
	paint::{Painter, TextStyle},
	util::{Geometry, WidgetRef},
	widgets::{Widget, WidgetState},
};
use cgmath::Vector2;
use skia_safe::{scalar, Point};
use crate::widgets::{Arrangements, Children, WidgetImpl};
use crate::widgets::leaf_widget::{LeafState, LeafWidget};

pub struct TextBlockWidgetState {
	leaf: LeafState,
	text: String,
	text_style: TextStyle,
}

pub type TextBlockWidget = WidgetImpl<TextBlockWidgetState>;

pub struct TextBlockWidgetBuilder(TextBlockWidget);

/// A Widget showing some simple text.
///
/// The desired size is the size the text would consume if the text were in one line.
impl TextBlockWidget {
	pub fn new() -> TextBlockWidgetBuilder {
		TextBlockWidgetBuilder(TextBlockWidgetState {
			leaf: Default::default(),
			text: String::default(),
			text_style: TextStyle::default(),
		}.into())
	}
}

impl TextBlockWidgetBuilder {
	pub fn text(mut self, text: String) -> Self {
		self.0.state_mut().text = text;
		self
	}

	pub fn build(self) -> WidgetRef<TextBlockWidget> {
		WidgetRef::new(self.0)
	}
}

impl Widget for TextBlockWidget {
	fn widget_state(&self) -> Ref<WidgetState> {
		self.widget_state(|v| &v.leaf.widget)
	}

	fn widget_state_mut(&self) -> RefMut<WidgetState> {
		self.widget_state_mut(|v| &mut v.leaf.widget)
	}

	fn paint(&self, _geometry: Geometry, layer: i32, painter: &mut Painter) -> i32 {
		let state = self.state();
		painter.draw_str(
			&state.text,
			Point::new(0.0, 0.0),
			&state.text_style.font,
			&state.text_style.color,
		);
		layer
	}

	fn get_desired_size(&self) -> Vector2<scalar> {
		let state = self.state();
		let (_, rect) = state
			.text_style
			.font
			.measure_str(&state.text, Some(&state.text_style.color));
		Vector2::new(rect.height(), rect.width())
	}

	fn get_children(&self) -> Children {
		self.leaf_get_children()
	}

	fn arrange_children(&self, geometry: Geometry) {
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
	fn leaf_state(&self) -> Ref<LeafState> {
		self.widget_state(|v| &v.leaf)
	}

	fn leaf_state_mut(&self) -> RefMut<LeafState> {
		self.widget_state_mut(|v| &mut v.leaf)
	}
}
