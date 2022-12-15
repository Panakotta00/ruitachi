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
		for (i, child) in self.get_children().enumerate() {
			let mut child_geo = geometry.clone();
			let pos = Vector2::new(width_step * i as scalar, 0.0);
			let size = Vector2::new(width_step, child_geo.local_size().y);
			list.push(child_geo.child_widget(child.clone(), pos, size));
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
