use crate::{paint::Painter, util::Geometry, widgets::Widget};

use skia_safe::Vector;
use crate::widgets::{Arrangements, WidgetArrangement, WidgetState};

#[derive(Default)]
pub struct PanelState {
	pub widget: WidgetState,
	arranged_children: Vec<WidgetArrangement>,
	cached_geometry: Geometry,
}

pub trait PanelWidget: Widget {
	fn panel_state(&self) -> &PanelState;
	fn panel_state_mut(&mut self) -> &mut PanelState;

	/// Gets called by the panels implementation of [arrange_children()] to get the list of newly
	/// arranged widgets.
	fn rearrange_children(&self, geometry: Geometry) -> Vec<WidgetArrangement>;

	/// Panel implementation of [Widget::paint()]
	fn panel_paint(&self, geometry: Geometry, mut layer: i32, painter: &mut Painter) -> i32 {
		let children = self.get_arranged_children();
		for child in children {
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

	/// Arranges the widget's children and caches them.
	///
	/// Useful implementation of [Widget.arrange_children()] for a layout only widget
	/// that only has slotted children.
	///
	/// # Default Implementation
	/// Arranges the children and stores the new arrangement as well as the new geometry in the widget state.
	fn panel_arrange_children(&mut self, geometry: Geometry){
		let widgets = self.rearrange_children(geometry);
		for widget in &widgets {
			widget.widget.get().arrange_children(widget.geometry);
		}
		let state = self.panel_state_mut();
		state.cached_geometry = geometry;
		state.arranged_children = widgets;
	}

	/// Panel Implementation of [Widget::get_arranged_children()]
	///
	/// # Default Implementation
	/// Returns the stored widget arrangement in the panel state
	fn panel_get_arranged_children(&self) -> Arrangements {
		let state = self.panel_state();
		state.arranged_children.iter().cloned().collect()
	}

	/// Panel Implementation of [Widget::cached_geometry()]
	///
	/// # Default Implementation
	/// Returns the stored cached geometry in the panel state
	fn panel_cached_geometry(&self) -> Geometry {
		let state = self.panel_state();
		state.cached_geometry
	}
}
