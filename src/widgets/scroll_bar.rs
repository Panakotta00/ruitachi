use crate::events::{Reply, WidgetEvent};
use crate::paint::Painter;
use crate::util::{Geometry, WidgetRef};
use crate::widgets::{Axis, Widget, WidgetState};
use cgmath::Vector2;
use skia_bindings::SkParsePath_PathEncoding::Relative;
use skia_bindings::SkRRect_Type::Rect;
use skia_safe::{scalar, Color, Color4f};
use std::ops::Range;

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

pub struct ScrollBarWidget {
	widget: WidgetState,
	direction: Axis,
	range: Range<f64>,
	value: f64,
	handle_size: ScrollBarHandleSize,
	handle: skia_safe::Paint,
	tray: skia_safe::Paint,
	drag_start: Option<(f64, Vector2<scalar>)>,
}

pub struct ScrollBarWidgetBuilder(ScrollBarWidget);

impl ScrollBarWidget {
	pub fn new() -> ScrollBarWidgetBuilder {
		ScrollBarWidgetBuilder(ScrollBarWidget {
			widget: Default::default(),
			direction: Axis::Vertical,
			range: 0.0..100.0,
			value: 0.0,
			handle_size: ScrollBarHandleSize::Fraction(0.1),
			handle: skia_safe::Paint::new(Color4f::from(Color::BLUE), None),
			tray: skia_safe::Paint::new(Color4f::from(Color::RED), None),
			drag_start: None,
		})
	}

	pub fn value(&self) -> f64 {
		self.value
	}

	pub fn set_value(&mut self, value: f64) {
		self.value = value.clamp(0.0, 1.0);
	}
}

impl ScrollBarWidgetBuilder {
	pub fn direction(mut self, direction: Axis) -> Self {
		self.0.direction = direction;
		self
	}

	pub fn build(mut self) -> WidgetRef<ScrollBarWidget> {
		WidgetRef::new(self.0)
	}
}

impl Widget for ScrollBarWidget {
	fn widget_state(&self) -> &WidgetState {
		&self.widget
	}

	fn widget_state_mut(&mut self) -> &mut WidgetState {
		&mut self.widget
	}

	fn paint(&self, geometry: Geometry, layer: i32, painter: &mut Painter) -> i32 {
		let size = self.handle_size.get_size(&self.range);
		let length = self.range.end - self.range.start;
		let local_size = geometry.local_size();
		let (size_direct, size_inv) = self.direction.get_vec_axis(local_size);
		let ppv = size_direct as f64 / (length + size) as f64;
		painter.draw_rect(
			skia_safe::Rect::new(0.0, 0.0, local_size.x, local_size.y),
			&self.tray,
		);
		let tl = self
			.direction
			.create_vec((length * self.value * ppv) as scalar, 0.0);
		let br = self
			.direction
			.create_vec(((length * self.value + size) * ppv) as scalar, size_inv);
		painter.draw_rect(skia_safe::Rect::new(tl.x, tl.y, br.x, br.y), &self.handle);
		layer + 1
	}

	fn get_desired_size(&self) -> Vector2<scalar> {
		Vector2::new(10.0, 10.0)
	}

	fn on_event(&mut self, event: &WidgetEvent) -> Reply {
		match event {
			WidgetEvent::OnCursorMove { pos, .. } => {
				if let Some(start) = self.drag_start {
					let local_size = self.cached_geometry().local_size();
					let local_axis = self.direction.get_vec_axis(local_size).0 as f64;
					let handle_size = self.handle_size.get_size(&self.range);
					let length = self.range.end - self.range.start;
					let value_per_local = (length + handle_size) / local_axis;

					let diff = self.direction.get_vec_axis(pos - start.1).0 as f64;
					self.set_value(diff * value_per_local / length + start.0);
					println!("Scroll: {}", self.value());
					Reply::handled()
				} else {
					Reply::unhandled()
				}
			}
			WidgetEvent::OnMouseButtonDown { mouse, pos, .. } => {
				self.drag_start = Some((self.value, *pos));
				Reply::handled().capture_cursor(*mouse)
			}
			WidgetEvent::OnMouseButtonUp { mouse, .. } => {
				self.drag_start = None;
				Reply::handled().release_cursor(*mouse)
			}
			_ => Reply::unhandled(),
		}
	}
}
