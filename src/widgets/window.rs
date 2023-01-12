use crate::paint::Painter;
use crate::util::{Geometry, WidgetRef};
use crate::widgets::{Widget, WidgetArrangement, WidgetState};
use cgmath::Vector2;
use rand::Rng;
use skia_safe::{scalar, Color, Paint, Rect};

pub trait Window: Widget {
	fn draw(&mut self, painter: &mut Painter) {
		let canvas = painter.canvas();
		canvas.clear(skia_safe::Color::DARK_GRAY);

		let geometry = Geometry::new(
			Vector2::new(0.0, 0.0),
			Vector2::new(painter.width() as scalar, painter.height() as scalar),
			Vector2::new(0.0, 0.0),
			Vector2::new(1.0, 1.0),
		);

		self.paint(geometry, 0, painter);
	}
}

pub struct WindowWidget {
	widget: WidgetState,
	content: Option<WidgetRef<dyn Widget>>,
}

pub struct WindowWidgetBuilder(WindowWidget);

impl WindowWidget {
	pub fn new(content: Option<WidgetRef<dyn Widget>>) -> WindowWidgetBuilder {
		WindowWidgetBuilder(WindowWidget {
			widget: WidgetState::default(),
			content,
		})
	}
}

impl WindowWidgetBuilder {
	pub fn build(self) -> WidgetRef<WindowWidget> {
		WidgetRef::new(self.0)
	}
}

impl Window for WindowWidget {}

impl Widget for WindowWidget {
	fn widget_state(&self) -> &WidgetState {
		&self.widget
	}

	fn widget_state_mut(&mut self) -> &mut WidgetState {
		&mut self.widget
	}

	fn paint(&self, geometry: Geometry, layer: i32, painter: &mut Painter) -> i32 {
		if let Some(content) = &self.content {
			content.get().paint(geometry, 0, painter)
		} else {
			layer
		}
	}

	fn arrange_children(&self, geometry: Geometry) -> Vec<WidgetArrangement> {
		if let Some(content) = &self.content {
			vec![WidgetArrangement::new(content.clone(), geometry)]
		} else {
			vec![]
		}
	}
}
