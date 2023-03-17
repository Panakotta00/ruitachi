use std::cell::{Ref, RefMut};
use crate::{
	paint::Painter,
	util::{Geometry, WidgetRef},
	widgets::{
		Children, HorizontalAlignment, PanelWidget, VerticalAlignment, Widget, WidgetArrangement,
		WidgetState,
	},
};
use cgmath::Vector2;
use skia_safe::scalar;
use crate::widgets::{Arrangements, PanelState};

pub struct BoxPanelState {
	panel: PanelState,
	child: WidgetRef<dyn Widget>,
	v_align: VerticalAlignment,
	h_align: HorizontalAlignment,
	override_x: Option<scalar>,
	override_y: Option<scalar>,
}

pub type BoxPanel = crate::widgets::WidgetImpl<BoxPanelState>;

pub struct BoxPanelBuilder(BoxPanel);

impl BoxPanelBuilder {
	pub fn v_align(mut self, v_align: VerticalAlignment) -> Self {
		self.0.state_mut().v_align = v_align;
		self
	}

	pub fn h_align(mut self, h_align: HorizontalAlignment) -> Self {
		self.0.state_mut().h_align = h_align;
		self
	}

	pub fn override_x(mut self, size_x: scalar) -> Self {
		self.0.state_mut().override_x = Some(size_x);
		self
	}

	pub fn override_y(mut self, size_y: scalar) -> Self {
		self.0.state_mut().override_y = Some(size_y);
		self
	}

	pub fn override_size(mut self, size: Vector2<scalar>) -> Self {
		let mut state = self.0.state_mut();
		state.override_x = Some(size.x);
		state.override_y = Some(size.y);
		drop(state);
		self
	}

	pub fn build(self) -> WidgetRef<BoxPanel> {
		WidgetRef::new(self.0)
	}
}

impl BoxPanel {
	pub fn new(child: WidgetRef<dyn Widget>) -> BoxPanelBuilder {
		BoxPanelBuilder(BoxPanelState {
			panel: Default::default(),
			child,
			v_align: VerticalAlignment::Top,
			h_align: HorizontalAlignment::Left,
			override_x: None,
			override_y: None,
		}.into())
	}
}

impl Widget for BoxPanel {
	fn widget_state(&self) -> Ref<WidgetState> {
		self.widget_state(|v| &v.panel.widget)
	}

	fn widget_state_mut(&mut self) -> RefMut<WidgetState> {
		self.widget_state_mut(|v| &mut v.panel.widget)
	}

	fn paint(&self, geometry: Geometry, layer: i32, painter: &mut Painter) -> i32 {
		self.panel_paint(geometry, layer, painter)
	}

	fn get_desired_size(&self) -> Vector2<scalar> {
		let state = self.state();
		let mut size = state.child.get().get_desired_size();
		if let Some(x) = state.override_x {
			size.x = x;
		}
		if let Some(y) = state.override_y {
			size.y = y;
		}
		size
	}

	fn get_children(&self) -> Children {
		Vec::new()
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

impl PanelWidget for BoxPanel {
	fn panel_state(&self) -> Ref<PanelState> {
		self.widget_state(|v| &v.panel)
	}

	fn panel_state_mut(&mut self) -> RefMut<PanelState> {
		self.widget_state_mut(|v| &mut v.panel)
	}

	fn rearrange_children(&self, geometry: Geometry) -> Vec<WidgetArrangement> {
		let state = self.state();
		let mut size = self.get_desired_size();
		let mut pos = Vector2::new(0.0, 0.0);
		match state.v_align {
			VerticalAlignment::Top => {}
			VerticalAlignment::Center => {
				pos.y = (geometry.local_size().y - size.y) / 2.0;
			}
			VerticalAlignment::Bottom => {
				pos.y = geometry.local_size().y - size.y;
			}
			VerticalAlignment::Fill => {
				size.y = geometry.local_size().y;
			}
		}
		match state.h_align {
			HorizontalAlignment::Left => {}
			HorizontalAlignment::Center => {
				pos.x = (geometry.local_size().x - size.x) / 2.0;
			}
			HorizontalAlignment::Right => {
				pos.x = geometry.local_size().x - size.x;
			}
			HorizontalAlignment::Fill => {
				size.x = geometry.local_size().x;
			}
		}
		vec![geometry.child_widget(state.child.clone(), pos, size)]
	}
}
