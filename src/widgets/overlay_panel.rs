use crate::paint::Painter;
use crate::util::{Geometry, WidgetRef};
use crate::widgets::{Children, PanelWidget, Widget, WidgetArrangement, WidgetState};
use cgmath::Vector2;
use skia_bindings::SkClipOp;
use skia_safe::{Rect, scalar};

pub struct OverlayPanelSlot {
	pub widget: WidgetRef<dyn Widget>,
}

pub struct OverlayPanel {
	widget: WidgetState,
	children: Vec<OverlayPanelSlot>,
}

pub struct OverlayPanelBuilder(OverlayPanel);

impl OverlayPanelBuilder {
	pub fn slot(mut self, widget: WidgetRef<dyn Widget>) -> Self {
		self.0.children.push(OverlayPanelSlot { widget });
		self
	}

	pub fn build(self) -> WidgetRef<OverlayPanel> {
		WidgetRef::new(self.0)
	}
}

impl OverlayPanel {
	pub fn new() -> OverlayPanelBuilder {
		OverlayPanelBuilder(OverlayPanel {
			widget: Default::default(),
			children: vec![],
		})
	}
}

impl Widget for OverlayPanel {
	fn widget_state(&self) -> &WidgetState {
		&self.widget
	}

	fn widget_state_mut(&mut self) -> &mut WidgetState {
		&mut self.widget
	}

	fn paint(&self, geometry: Geometry, layer: i32, painter: &mut Painter) -> i32 {
		painter.clip_rect(Rect::new(
			0.0,
			0.0,
			geometry.local_size().x,
			geometry.local_size().y), Some(SkClipOp::Intersect), None);
		PanelWidget::paint(self, geometry, layer, painter)
	}

	fn get_desired_size(&self) -> Vector2<scalar> {
		let mut size = Vector2::<scalar>::new(0.0, 0.0);
		for child in &self.children {
			let desire = child.widget.get().get_desired_size();
			size.x = size.x.max(desire.x);
			size.y = size.y.max(desire.y);
		}
		size
	}

	fn get_children(&self) -> Children<'_> {
		Box::new(self.children.iter().map(|child| &child.widget))
	}

	fn arrange_children(&self, geometry: Geometry) -> Vec<WidgetArrangement> {
		self.children
			.iter()
			.map(|slot| {
				let pos = Vector2::new(0.0, 0.0);
				let size = geometry.local_size();
				geometry.child_widget(slot.widget.clone(), pos, size)
			})
			.collect()
	}
}

impl PanelWidget for OverlayPanel {}
