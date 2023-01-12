use crate::paint::Painter;
use crate::util::{Geometry, WidgetRef};
use crate::widgets::layout::Growth;
use crate::widgets::{Children, PanelWidget, Widget, WidgetArrangement, WidgetState};
use cgmath::Vector2;
use skia_safe::scalar;
use std::process::Child;

pub enum LinearPanelDirection {
	Vertical,
	Horizontal,
}

pub struct LinearPanelSlot {
	pub widget: WidgetRef<dyn Widget>,
	pub growth: Growth,
}

pub struct LinearPanel {
	widget: WidgetState,
	direction: LinearPanelDirection,
	children: Vec<LinearPanelSlot>,
}

pub struct LinearPanelBuilder(LinearPanel);

impl LinearPanelBuilder {
	pub fn slot(mut self, widget: WidgetRef<dyn Widget>, growth: Growth) -> Self {
		self.0.children.push(LinearPanelSlot { widget, growth });
		self
	}

	pub fn build(self) -> WidgetRef<LinearPanel> {
		WidgetRef::new(self.0)
	}
}

impl LinearPanel {
	pub fn new(direction: LinearPanelDirection) -> LinearPanelBuilder {
		LinearPanelBuilder(LinearPanel {
			widget: Default::default(),
			direction,
			children: vec![],
		})
	}

	#[inline]
	fn get_dir_val<'a>(&self, vector: &'a Vector2<scalar>) -> &'a scalar {
		match self.direction {
			LinearPanelDirection::Vertical => &vector.y,
			LinearPanelDirection::Horizontal => &vector.x,
		}
	}

	#[inline]
	fn get_dir_val_mut<'a>(&self, vector: &'a mut Vector2<scalar>) -> &'a mut scalar {
		match self.direction {
			LinearPanelDirection::Vertical => &mut vector.y,
			LinearPanelDirection::Horizontal => &mut vector.x,
		}
	}
}

impl Widget for LinearPanel {
	fn widget_state(&self) -> &WidgetState {
		&self.widget
	}

	fn widget_state_mut(&mut self) -> &mut WidgetState {
		&mut self.widget
	}

	fn paint(&self, geometry: Geometry, layer: i32, painter: &mut Painter) -> i32 {
		PanelWidget::paint(self, geometry, layer, painter)
	}

	fn get_desired_size(&self) -> Vector2<scalar> {
		let mut size = Vector2::<scalar>::new(0.0, 0.0);
		for child in &self.children {
			let desire = child.widget.get().get_desired_size();
			match self.direction {
				LinearPanelDirection::Vertical => {
					size.x = size.x.max(desire.x);
					size.y += desire.y;
				}
				LinearPanelDirection::Horizontal => {
					size.x += desire.x;
					size.y = size.y.max(desire.y);
				}
			}
		}
		size
	}

	fn get_children(&self) -> Children<'_> {
		Box::new(self.children.iter().map(|child| &child.widget))
	}

	fn arrange_children(&self, geometry: Geometry) -> Vec<WidgetArrangement> {
		let mut list = Vec::new();
		let width_step = self.get_dir_val(&geometry.local_size()) / self.children.len() as scalar;
		let mut fit = Vec::new();
		let mut value = Vec::new();
		let mut fill = Vec::new();
		let mut required_width = 0.0;
		let mut sum_value = 0.0;
		for (i, child) in self.children.iter().enumerate() {
			match child.growth {
				Growth::Fill => fill.push(&child.widget),
				Growth::Fit => fit.push(&child.widget),
				Growth::Val(val) => {
					value.push(&child.widget);
					sum_value += val;
				}
			}
			required_width += self.get_dir_val(&child.widget.get().get_desired_size());
		}
		let available_width = self.get_dir_val(&geometry.local_size()) - required_width;

		if available_width <= 0.0 {
			// If there is not enough space for all widgets, desired size,
			// fit all to fit as many as possible
			fit.append(&mut fill);
			fit.append(&mut value);
		} else if fill.len() > 0 {
			// If there is at least one slot that fills the panel,
			// then there cannot be any slots scaled by a value
			fit.append(&mut value);
		}

		let sized_fitted = value.len() + fill.len() <= 0;

		let mut last_offset = 0.0;
		for (i, child) in self.children.iter().enumerate() {
			let desired_width = *self.get_dir_val(&child.widget.get().get_desired_size());
			let mut size = geometry.local_size();
			let width = if fit.contains(&&child.widget) {
				desired_width
					+ if sized_fitted {
						available_width / fit.len() as scalar
					} else {
						0.0
					}
			} else if value.contains(&&child.widget) {
				if let Growth::Val(val) = child.growth {
					desired_width + available_width * (val / sum_value)
				} else {
					0.0
				}
			} else if fill.contains(&&child.widget) {
				desired_width + available_width / fill.len() as scalar
			} else {
				0.0
			};
			*self.get_dir_val_mut(&mut size) = width;
			let mut pos = Vector2::new(0.0, 0.0);
			*self.get_dir_val_mut(&mut pos) = last_offset;
			list.push(geometry.child_widget(child.widget.clone(), pos, size));
			last_offset += width;
		}
		list
	}
}

impl PanelWidget for LinearPanel {}
