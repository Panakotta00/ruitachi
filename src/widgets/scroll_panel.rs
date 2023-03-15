use crate::{
	paint::Painter,
	util::{scalar, Geometry, WidgetRef},
	widgets::{
		Axis, Axis::Vertical, PanelWidget, ScrollBarWidget, Widget, WidgetArrangement, WidgetState,
	},
};
use cgmath::Vector2;
use skia_bindings::SkClipOp;
use skia_safe::{Rect, Vector};

#[derive(Eq, PartialEq, Copy, Clone)]
pub enum ScrollPanelDirection {
	Horizontal,
	Vertical,
	Both,
}

struct ScrollPanelCache {
	arranged_content: Option<WidgetArrangement>,
	arranged_decorations: Vec<WidgetArrangement>,
}

pub struct ScrollPanel {
	widget: WidgetState,
	direction: ScrollPanelDirection,
	horizontal: Option<WidgetRef<ScrollBarWidget>>,
	vertical: Option<WidgetRef<ScrollBarWidget>>,
	content: Option<WidgetRef<dyn Widget>>,
	cache: WidgetRef<ScrollPanelCache>,
}

impl ScrollPanel {
	pub fn new() -> ScrollPanelBuilder {
		ScrollPanelBuilder(Self {
			widget: Default::default(),
			direction: ScrollPanelDirection::Both,
			horizontal: None,
			vertical: None,
			content: None,
			cache: WidgetRef::new(ScrollPanelCache {
				arranged_decorations: Default::default(),
				arranged_content: None,
			}),
		})
	}
}

pub struct ScrollPanelBuilder(ScrollPanel);

impl ScrollPanelBuilder {
	pub fn build(self) -> WidgetRef<ScrollPanel> {
		WidgetRef::new(self.0)
	}

	pub fn direction(mut self, direction: ScrollPanelDirection) -> Self {
		self.0.direction = direction;
		if direction == ScrollPanelDirection::Vertical || direction == ScrollPanelDirection::Both {
			self.0.vertical = Some(ScrollBarWidget::new().direction(Axis::Vertical).build());
		}
		if direction == ScrollPanelDirection::Horizontal || direction == ScrollPanelDirection::Both
		{
			self.0.horizontal = Some(ScrollBarWidget::new().direction(Axis::Horizontal).build());
		}
		self
	}

	pub fn content(mut self, content: WidgetRef<dyn Widget>) -> Self {
		self.0.content = Some(content);
		self
	}
}

impl Widget for ScrollPanel {
	fn widget_state(&self) -> &WidgetState {
		&self.widget
	}

	fn widget_state_mut(&mut self) -> &mut WidgetState {
		&mut self.widget
	}

	fn arrange_children(&self, geometry: Geometry) -> Vec<WidgetArrangement> {
		let mut arranged = Vec::new();

		let mut horizontal = false;
		let mut vertical = false;
		let desired_size = match &self.content {
			Some(content) => content.get().get_desired_size(),
			None => Vector2::new(0.0, 0.0),
		};
		let available_size = geometry.local_size();

		let mut available_content_size = Vector2::new(
			match &self.vertical {
				Some(scroll_bar) => available_size.x - scroll_bar.get().get_desired_size().x,
				None => available_size.x,
			},
			match &self.horizontal {
				Some(scroll_bar) => available_size.y - scroll_bar.get().get_desired_size().y,
				None => available_size.y,
			},
		);

		let overflow_size = desired_size - available_content_size;
		let overflow_size = Vector2::new(
			match overflow_size.x {
				v @ 0.0.. => {
					horizontal = self.horizontal.is_some();
					v
				}
				_ => 0.0,
			},
			match overflow_size.y {
				v @ 0.0.. => {
					vertical = self.vertical.is_some();
					v
				}
				_ => 0.0,
			},
		);

		if let Some(content) = &self.content {
			let pos = Vector2::new(
				match horizontal {
					true => {
						self.horizontal.as_ref().unwrap().get().value() as scalar * -overflow_size.x
					}
					false => 0.0,
				},
				match vertical {
					true => {
						self.vertical.as_ref().unwrap().get().value() as scalar * -overflow_size.y
					}
					false => 0.0,
				},
			);
			let size = content.get().get_desired_size();
			let size = Vector2::new(
				match geometry.local_size().x {
					x if x > size.x => x,
					_ => size.x,
				},
				match geometry.local_size().y {
					y if y > size.y => y,
					_ => size.y,
				},
			);
			let child = geometry.child_widget(content.clone(), pos, size);
			arranged.push(child.clone());
			self.cache.get().arranged_content = Some(child);
		}

		self.cache.get().arranged_decorations.clear();
		if vertical {
			let scroll = self.vertical.as_ref().unwrap();
			scroll.get().set_range(0.0..overflow_size.y as f64);
			let pos = Vector2::new(available_content_size.x, 0.0);
			let size = Vector2::new(scroll.get().get_desired_size().x, available_content_size.y);
			let child = geometry.child_widget(scroll.clone(), pos, size);
			arranged.push(child.clone());
			self.cache.get().arranged_decorations.push(child);
		}
		if horizontal {
			let scroll = self.horizontal.as_ref().unwrap();
			scroll.get().set_range(0.0..overflow_size.x as f64);
			let pos = Vector2::new(0.0, available_content_size.y);
			let size = Vector2::new(available_content_size.x, scroll.get().get_desired_size().y);
			let child = geometry.child_widget(scroll.clone(), pos, size);
			arranged.push(child.clone());
			self.cache.get().arranged_decorations.push(child);
		}

		arranged
	}

	fn paint(&self, geometry: Geometry, mut layer: i32, painter: &mut Painter) -> i32 {
		self.arrange_children(geometry);
		if let Some(content) = &self.cache.get().arranged_content {
			painter.save();
			painter.clip_rect(
				Rect::new(0.0, 0.0, geometry.local_size().x, geometry.local_size().y),
				Some(SkClipOp::Intersect),
				None,
			);
			painter.translate(Vector::new(
				content.geometry.local_pos().x,
				content.geometry.local_pos().y,
			));
			content.widget.get().paint(content.geometry, layer, painter);
			painter.restore();
			layer += 1;
		}
		for child in self.cache.get().arranged_decorations.iter() {
			painter.save();
			painter.translate(Vector::new(
				child.geometry.local_pos().x,
				child.geometry.local_pos().y,
			));
			child.widget.get().paint(child.geometry, layer, painter);
			painter.restore();
			layer += 1;
		}
		layer
	}
}

impl PanelWidget for ScrollPanel {}
