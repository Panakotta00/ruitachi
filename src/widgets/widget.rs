use crate::events::{MouseButtonEvent, PointerEvent, Reply, WidgetEvent};
use crate::paint::Painter;
use crate::util::{Geometry, WidgetRef};
use cgmath::Vector2;
use rand::Rng;
use skia_safe::{scalar, Color, Color4f, Paint, PixelGeometry, Rect};
use std::cmp::max;

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
	parent: Option<WidgetRef<dyn Widget>>,
	cached_geometry: Geometry,
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

	fn on_event(&mut self, event: &WidgetEvent) -> Reply {
		Reply::unhandled()
	}
}
