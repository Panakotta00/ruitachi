use crate::{paint::Painter, util::Geometry, widgets::Widget};

use skia_safe::Vector;

pub trait PanelWidget: Widget {
	fn paint(&self, geometry: Geometry, mut layer: i32, painter: &mut Painter) -> i32 {
		let children = self.arrange_children(geometry);
		for child in children.iter() {
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
