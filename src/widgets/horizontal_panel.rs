use std::process::Child;
use crate::paint::Painter;
use crate::util::{Geometry, WidgetRef};
use crate::widgets::{Children, PanelWidget, Widget, WidgetArrangement, WidgetState};
use cgmath::Vector2;
use skia_safe::scalar;
use crate::widgets::layout::Growth;

pub struct HorizontalPanelWidgetSlot {
	pub widget: WidgetRef<dyn Widget>,
	pub growth: Growth,
}

#[derive(Default)]
pub struct HorizontalPanelWidget {
	widget: WidgetState,
	children: Vec<HorizontalPanelWidgetSlot>,
}

pub struct HorizontalPanelWidgetBuilder(HorizontalPanelWidget);

impl HorizontalPanelWidget {
	pub fn new() -> HorizontalPanelWidgetBuilder {
		HorizontalPanelWidgetBuilder(HorizontalPanelWidget::default())
	}
}

impl HorizontalPanelWidgetBuilder {
	pub fn slot(mut self, widget: WidgetRef<dyn Widget>, growth: Growth) -> Self {
		self.0.children.push(HorizontalPanelWidgetSlot{widget, growth});
		self
	}

	pub fn build(self) -> WidgetRef<HorizontalPanelWidget> {
		WidgetRef::new(self.0)
	}
}

impl Widget for HorizontalPanelWidget {
	fn widget_state(&self) -> &WidgetState {
		&self.widget
	}

	fn widget_state_mut(&mut self) -> &mut WidgetState {
		&mut self.widget
	}

	fn paint(&self, geometry: Geometry, layer: i32, painter: &mut Painter) -> i32 {
		PanelWidget::paint(self, geometry, layer, painter)
	}

	fn get_children(&self) -> Children<'_> {
		Box::new(self.children.iter().map(|child| &child.widget))
	}

	fn arrange_children(&self, geometry: Geometry) -> Vec<WidgetArrangement> {
		let mut list = Vec::new();
		let width_step = geometry.local_size().x / self.children.len() as scalar;
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
				},
			}
			required_width += child.widget.get().get_desired_size().x;
		}
		let available_width = geometry.local_size().x - required_width;

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
			let size = Vector2::new(
				if fit.contains(&&child.widget) {
					child.widget.get().get_desired_size().x + if sized_fitted {
						available_width / fit.len() as scalar
					} else {
						0.0
					}
				} else if value.contains(&&child.widget) {
					if let Growth::Val(val) = child.growth {
						child.widget.get().get_desired_size().x + available_width * (val / sum_value)
					} else {
						0.0
					}
				} else if fill.contains(&&child.widget) {
					child.widget.get().get_desired_size().x + available_width / fill.len() as scalar
				} else {
					0.0
				},
				geometry.local_size().y);
			let pos = Vector2::new(last_offset, 0.0);
			list.push(geometry.child_widget(child.widget.clone(), pos, size));
			last_offset += size.x;
		}
		list
	}

	fn get_desired_size(&self) -> Vector2<scalar> {
		let mut size = Vector2::<scalar>::new(0.0, 0.0);
		for child in &self.children {
			let desire = child.widget.get().get_desired_size();
			size.x += desire.x;
			size.y = size.y.max(desire.y);
		}
		size
	}
}

impl PanelWidget for HorizontalPanelWidget {}
