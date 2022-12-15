use crate::paint::Painter;
use crate::util::Geometry;
use crate::widgets::Widget;

pub trait PanelWidget: Widget {
	fn paint(&self, geometry: Geometry, mut layer: i32, painter: &mut Painter) -> i32 {
		let children = self.arrange_children(geometry);
		for child in children.iter() {
			child.widget.get().paint(child.geometry, layer, painter);
			layer += 1;
		}
		layer
	}
}
