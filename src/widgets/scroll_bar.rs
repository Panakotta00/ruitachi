use std::cell::{Ref, RefMut};
use std::ops::Range;
use crate::{
	events::{Reply, WidgetEvent},
	paint::Painter,
	util::{Geometry, WidgetRef},
	widgets::{Axis, Widget, WidgetState},
};
use cgmath::Vector2;
use skia_safe::{scalar, Color, Color4f};
use crate::widgets::{Arrangements, Children, PanelState, WidgetArrangement, WidgetImpl};
use crate::widgets::leaf_widget::{LeafState, LeafWidget};

pub enum ScrollBarHandleSize {
	Absolute(f64),
	Fraction(f64),
}

impl ScrollBarHandleSize {
	pub fn get_size(&self, range: &Range<f64>) -> f64 {
		match *self {
			ScrollBarHandleSize::Absolute(size) => size,
			ScrollBarHandleSize::Fraction(fraction) => fraction * (range.end - range.start),
		}
	}
}

pub struct ScrollBarWidgetState {
	leaf: LeafState,
	direction: Axis,
	range: Range<f64>,
	value: f64,
	handle_size: ScrollBarHandleSize,
	handle: skia_safe::Paint,
	tray: skia_safe::Paint,
	drag_start: Option<(f64, Vector2<scalar>)>,
	pub on_value_changed: Option<Box<dyn Fn(f64, f64)>>,
}

pub type ScrollBarWidget = WidgetImpl<ScrollBarWidgetState>;

pub struct ScrollBarWidgetBuilder(ScrollBarWidget);

impl ScrollBarWidget {
	pub fn new() -> ScrollBarWidgetBuilder {
		ScrollBarWidgetBuilder(ScrollBarWidgetState {
			leaf: Default::default(),
			direction: Axis::Vertical,
			range: 0.0..100.0,
			value: 0.0,
			handle_size: ScrollBarHandleSize::Fraction(0.1),
			handle: skia_safe::Paint::new(Color4f::from(Color::BLUE), None),
			tray: skia_safe::Paint::new(Color4f::from(Color::RED), None),
			drag_start: None,
			on_value_changed: None,
		}.into())
	}

	pub fn value(&self) -> f64 {
		self.state().value
	}

	pub fn set_value(&self, value: f64) {
		let old_value = self.state().value;
		let new_value = value.clamp(0.0, 1.0);
		self.state_mut().value = new_value;
		let delegate = self.state_mut().on_value_changed.take();
		if let Some(delegate) = delegate {
			delegate(old_value, new_value);
			self.state_mut().on_value_changed = Some(delegate);
		}
	}

	pub fn set_range(&self, range: Range<f64>) {
		self.state_mut().range = range;
		let value = self.state().value;
		self.set_value(value);
	}
}

impl ScrollBarWidgetBuilder {
	pub fn direction(self, direction: Axis) -> Self {
		self.0.state_mut().direction = direction;
		self
	}

	pub fn build(self) -> WidgetRef<ScrollBarWidget> {
		WidgetRef::new(self.0)
	}
}

impl Widget for ScrollBarWidget {
	fn widget_state(&self) -> Ref<WidgetState> {
		self.widget_state(|v| &v.leaf.widget)
	}

	fn widget_state_mut(&self) -> RefMut<WidgetState> {
		self.widget_state_mut(|v| &mut v.leaf.widget)
	}

	fn paint(&self, geometry: Geometry, layer: i32, painter: &mut Painter) -> i32 {
		let state = self.state();
		let size = state.handle_size.get_size(&state.range);
		let length = state.range.end - state.range.start;
		let local_size = geometry.local_size();
		let (size_direct, size_inv) = state.direction.get_vec_axis(local_size);
		let ppv = size_direct as f64 / (length + size) as f64;
		painter.draw_rect(
			skia_safe::Rect::new(0.0, 0.0, local_size.x, local_size.y),
			&state.tray,
		);
		let tl = state
			.direction
			.create_vec((length * state.value * ppv) as scalar, 0.0);
		let br = state
			.direction
			.create_vec(((length * state.value + size) * ppv) as scalar, size_inv);
		painter.draw_rect(skia_safe::Rect::new(tl.x, tl.y, br.x, br.y), &state.handle);
		layer + 1
	}

	fn get_desired_size(&self) -> Vector2<scalar> {
		Vector2::new(10.0, 10.0)
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

	fn on_event(&self, event: &WidgetEvent) -> Reply {
		match event {
			WidgetEvent::OnCursorMove { pos, .. } => {
				let state = self.state();
				if let Some(start) = state.drag_start {
					let local_size = self.cached_geometry().local_size();
					let local_axis = state.direction.get_vec_axis(local_size).0 as f64;
					let handle_size = state.handle_size.get_size(&state.range);
					let length = state.range.end - state.range.start;
					let value_per_local = (length + handle_size) / local_axis;
					let diff = state.direction.get_vec_axis(pos - start.1).0 as f64;
					drop(state);

					self.set_value(diff * value_per_local / length + start.0);

					Reply::handled()
				} else {
					Reply::unhandled()
				}
			}
			WidgetEvent::OnMouseButtonDown { mouse, pos, .. } => {
				let mut state = self.state_mut();
				state.drag_start = Some((state.value, *pos));
				Reply::handled().capture_cursor(*mouse)
			}
			WidgetEvent::OnMouseButtonUp { mouse, .. } => {
				self.state_mut().drag_start = None;
				Reply::handled().release_cursor(*mouse)
			}
			_ => Reply::unhandled(),
		}
	}

	fn cached_geometry(&self) -> Geometry {
		self.leaf_cached_geometry()
	}
}

impl LeafWidget for ScrollBarWidget {
	fn leaf_state(&self) -> Ref<LeafState> {
		self.widget_state(|v| &v.leaf)
	}

	fn leaf_state_mut(&self) -> RefMut<LeafState> {
		self.widget_state_mut(|v| &mut v.leaf)
	}
}
