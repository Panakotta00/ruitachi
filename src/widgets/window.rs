use std::cell::{Ref, RefMut};
use crate::{
	paint::Painter,
	util::{Geometry, WidgetRef, WindowId},
	widgets::{Widget, WidgetArrangement, WidgetState},
};
use cgmath::Vector2;

use skia_safe::scalar;
use crate::widgets::{Arrangements, Children, WidgetImpl};

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

pub struct WindowWidgetState {
	widget: WidgetState,
	window_id: Option<WindowId>,
	content: Option<WidgetRef<dyn Widget>>,
	cached_content: Option<WidgetArrangement>,
	cached_geometry: Geometry,
}

pub type WindowWidget = WidgetImpl<WindowWidgetState>;

pub struct WindowWidgetBuilder(WindowWidget);

impl WindowWidget {
	pub fn new(content: Option<WidgetRef<dyn Widget>>) -> WindowWidgetBuilder {
		WindowWidgetBuilder(WindowWidgetState {
			widget: WidgetState::default(),
			window_id: None,
			content,
			cached_content: None,
			cached_geometry: Default::default(),
		}.into())
	}
}

impl WindowWidgetBuilder {
	pub fn build(self) -> WidgetRef<WindowWidget> {
		WidgetRef::new(self.0)
	}
}

impl Window for WindowWidget {
	fn id(&self) -> Option<WindowId> {
		self.state().window_id
	}

	fn set_id(&mut self, id: Option<WindowId>) {
		self.state_mut().window_id = id;
	}
}

impl Widget for WindowWidget {
	fn widget_state(&self) -> Ref<WidgetState> {
		self.widget_state(|v| &v.widget)
	}

	fn widget_state_mut(&mut self) -> RefMut<WidgetState> {
		self.widget_state_mut(|v| &mut v.widget)
	}

	fn paint(&self, geometry: Geometry, layer: i32, painter: &mut Painter) -> i32 {
		if let Some(content) = self.state().content.clone() {
			content.get().paint(geometry, 0, painter)
		} else {
			layer
		}
	}

	fn get_desired_size(&self) -> Vector2<scalar> {
		if let Some(content) = &self.state().content {
			content.get().get_desired_size()
		} else {
			Vector2::new(0.0, 0.0)
		}
	}

	fn get_children(&self) -> Children {
		match &self.state().content {
			Some(content) => vec![content.clone()],
			None => vec![]
		}
	}

	fn arrange_children(&mut self, geometry: Geometry) {
		let content = self.state().content.clone();
		self.state_mut().cached_content = match content {
			Some(content) => {
				content.get().arrange_children(geometry);
				Some(WidgetArrangement::new(content, geometry))
			},
			None => None,
		}
	}

	fn get_arranged_children(&self) -> Arrangements {
		vec![self.state().cached_content.clone()].into_iter().filter_map(|v| v).collect()
	}

	fn cached_geometry(&self) -> Geometry {
		self.state().cached_geometry
	}
}
