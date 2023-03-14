use crate::{
	events::{Reply, WidgetEvent},
	paint::Painter,
	util::{Geometry, WidgetRef},
};
use cgmath::Vector2;

use skia_safe::scalar;

#[derive(Clone)]
pub struct WidgetArrangement {
	pub widget: WidgetRef<dyn Widget>,
	pub geometry: Geometry,
}

impl WidgetArrangement {
	pub fn new(widget: WidgetRef<dyn Widget>, geometry: Geometry) -> Self {
		Self { widget, geometry }
	}
}

pub type Children<'a> = Box<dyn Iterator<Item = &'a WidgetRef<dyn Widget>> + 'a>;

#[derive(Default)]
pub struct WidgetState {
	pub parent: Option<WidgetRef<dyn Widget>>,
	pub cached_geometry: Geometry,
	pub arranged_children: Vec<WidgetArrangement>,
}

pub trait Widget {
	fn widget_state(&self) -> &WidgetState;
	fn widget_state_mut(&mut self) -> &mut WidgetState;

	fn get_parent(&self) -> Option<WidgetRef<dyn Widget>> {
		self.widget_state().parent.clone()
	}

	fn set_parent(&mut self, parent: Option<WidgetRef<dyn Widget>>) {
		self.widget_state_mut().parent = parent;
	}

	fn paint(&self, geometry: Geometry, layer: i32, painter: &mut Painter) -> i32 {
		geometry;
		painter;
		layer
	}

	fn get_desired_size(&self) -> Vector2<scalar> {
		Vector2::new(0.0, 0.0)
	}

	fn get_children(&self) -> Children<'_> {
		Box::new(std::iter::empty())
	}

	fn arrange_children(&self, geometry: Geometry) -> Vec<WidgetArrangement> {
		Vec::new()
	}

	fn get_arranged_children(&self) -> &Vec<WidgetArrangement> {
		&self.widget_state().arranged_children
	}

	fn on_event(&mut self, _event: &WidgetEvent) -> Reply {
		Reply::unhandled()
	}

	fn cached_geometry(&self) -> Geometry {
		self.widget_state().cached_geometry
	}
}

impl dyn Widget {
	pub fn calculate_arrange_children(&mut self, geometry: Geometry) -> &Vec<WidgetArrangement> {
		let widgets = self.arrange_children(geometry);
		let state = self.widget_state_mut();
		state.cached_geometry = geometry;
		state.arranged_children = widgets;
		&state.arranged_children
	}
}
