use crate::{
	paint::Painter,
	util::{Geometry, WidgetRef, WindowId},
	widgets::{Widget, WidgetArrangement, WidgetState},
};
use cgmath::Vector2;

use skia_safe::scalar;

pub trait Window: Widget {
	fn draw(&mut self, canvas: &mut skia_safe::Canvas, size: (scalar, scalar)) {
		let geometry = Geometry::new(
			Vector2::new(0.0, 0.0),
			Vector2::new(size.0, size.1),
			Vector2::new(0.0, 0.0),
			Vector2::new(1.0, 1.0),
		);

		canvas.clear(skia_safe::Color::DARK_GRAY);
		canvas.save();
		self.paint(geometry, 0, canvas);
		canvas.restore();
	}

	fn id(&self) -> Option<WindowId>;
	fn set_id(&mut self, id: Option<WindowId>);
}

pub struct WindowWidget {
	widget: WidgetState,
	window_id: Option<WindowId>,
	content: Option<WidgetRef<dyn Widget>>,
}

pub struct WindowWidgetBuilder(WindowWidget);

impl WindowWidget {
	pub fn new(content: Option<WidgetRef<dyn Widget>>) -> WindowWidgetBuilder {
		WindowWidgetBuilder(WindowWidget {
			widget: WidgetState::default(),
			window_id: None,
			content,
		})
	}
}

impl WindowWidgetBuilder {
	pub fn build(self) -> WidgetRef<WindowWidget> {
		WidgetRef::new(self.0)
	}
}

impl Window for WindowWidget {
	fn id(&self) -> Option<WindowId> {
		self.window_id
	}

	fn set_id(&mut self, id: Option<WindowId>) {
		self.window_id = id;
	}
}

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
