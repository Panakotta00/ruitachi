use crate::{
	paint::Painter,
	util::{Geometry, WidgetRef},
	widgets::{Children, PanelWidget, Widget, WidgetArrangement, WidgetState},
};
use cgmath::Vector2;

use skia_safe::scalar;
use crate::widgets::{Arrangements, PanelState};

pub struct OverlayPanelSlot {
	pub widget: WidgetRef<dyn Widget>,
}

pub struct OverlayPanel {
	panel: PanelState,
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
			panel: Default::default(),
			children: vec![],
		})
	}
}

impl Widget for OverlayPanel {
	fn widget_state(&self) -> &WidgetState {
		&self.panel.widget
	}

	fn widget_state_mut(&mut self) -> &mut WidgetState {
		&mut self.panel.widget
	}

	fn paint(&self, geometry: Geometry, layer: i32, painter: &mut Painter) -> i32 {
		self.panel_paint(geometry, layer, painter)
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

	fn get_children(&self) -> Children {
		self.children.iter().map(|child| child.widget.clone()).collect()
	}

	fn arrange_children(&mut self, geometry: Geometry) {
		self.panel_arrange_children(geometry);
	}

	fn get_arranged_children(&self) -> Arrangements {
		self.panel_get_arranged_children()
	}

	fn cached_geometry(&self) -> Geometry {
		self.panel_cached_geometry()
	}
}

impl PanelWidget for OverlayPanel {
	fn panel_state(&self) -> &PanelState {
		&self.panel
	}

	fn panel_state_mut(&mut self) -> &mut PanelState {
		&mut self.panel
	}

	fn rearrange_children(&self, geometry: Geometry) -> Vec<WidgetArrangement> {
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
