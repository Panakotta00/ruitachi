mod events;

use cgmath::Vector2;
use skia_safe::scalar;
pub use events::*;
use crate::util::{Geometry, WidgetRef};
use crate::widgets::{Widget, Window};

pub struct Reply {
	handled: bool,
}

impl Reply {
	pub fn unhandled() -> Reply {
		Reply { handled: false }
	}

	pub fn handled() -> Reply {
		Reply { handled: true }
	}
}

pub enum WidgetEvent {
	OnMouseMove,
	OnMouseInput,
}

pub struct WidgetPath {
	pub widget: WidgetRef<dyn Widget>,
	pub children: Vec<Box<WidgetPath>>,
}

pub fn get_widget_path_under_position(geometry: Geometry, widget: WidgetRef<dyn Widget>, pos: &Vector2<scalar>) -> WidgetPath {
	let mut path = WidgetPath{widget: widget.clone(), children: Vec::new()};
	for child_arrangement in widget.get().arrange_children(geometry).iter().rev() {
		if !child_arrangement.geometry.contains_absolute_pos(pos) {
			continue
		}
		path.children.push(Box::new(get_widget_path_under_position(child_arrangement.geometry, child_arrangement.widget.clone(), pos)));
	}
	path
}

pub fn bubble_event(path: WidgetPath, event: &WidgetEvent) -> Reply {
	for child in path.children {
		let reply = bubble_event(*child, event);
		if reply.handled {
			return reply
		}
	}
	path.widget.get().on_event(event)
}