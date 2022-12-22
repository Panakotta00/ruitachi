mod events;

use std::collections::{HashMap, HashSet};
use std::rc::{Rc, Weak};
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
	OnMouseInput,
	OnMouseEnter,
	OnMouseMove,
	OnMouseLeave,
}

pub struct WidgetPath {
	pub widget: WidgetRef<dyn Widget>,
	pub children: Vec<WidgetPath>,
}

impl WidgetPath {
	pub fn bubble(&self) -> WidgetPathIteratorBubble {
		WidgetPathIteratorBubble {
			iter: Box::new(WidgetPathIteratorBubbleInternal {
				path: Some(&self),
				path_child_index: None,
				prev_iterator: None,
			}),
		}
	}
}

pub struct WidgetPathIteratorBubble<'a> {
	iter: Box<WidgetPathIteratorBubbleInternal<'a>>
}

struct WidgetPathIteratorBubbleInternal<'a> {
	path: Option<&'a WidgetPath>,
	path_child_index: Option<usize>,
	prev_iterator: Option<Box<WidgetPathIteratorBubbleInternal<'a>>>,
}

impl<'a> Iterator for WidgetPathIteratorBubble<'a> {
	type Item = &'a WidgetRef<dyn Widget>;
	fn next(&mut self) -> Option<&'a WidgetRef<dyn Widget>> {
		// go to deepest possible widget
		while let Some(path) = self.iter.path {
			let idx = match self.iter.path_child_index {
				None => 0,
				Some(idx) => idx + 1,
			};
			self.iter.path_child_index = Some(idx);

			if idx >= path.children.len() {
				// all children passed, now the widget it self
				break
			}

			// first or next children, so we need to wrap in new iterator for that child
			let mut next = Box::new(WidgetPathIteratorBubbleInternal {
				path: Some(&path.children[idx]),
				path_child_index: None,
				prev_iterator: None,
			});
			std::mem::swap(&mut self.iter, &mut next);
			self.iter.prev_iterator = Some(next);
		};

		let ret = self.iter.path.map(|p| &p.widget);

		// got back to prev, no need to go to next child, as next call would do that,
		// as well as passing the widget it self
		if let Some(idx) = self.iter.path_child_index {
			if self.iter.prev_iterator.is_none() {
				// no prev iterator, means, end of iterator, tell iterator to end by setting path to None
				self.iter.path = None
			} else {
				let mut prev = None;
				std::mem::swap(&mut prev, &mut self.iter.prev_iterator);
				self.iter = prev.unwrap();
			}
		}

		ret
	}
}

pub fn get_widget_path_under_position(geometry: Geometry, widget: WidgetRef<dyn Widget>, pos: &Vector2<scalar>) -> WidgetPath {
	let mut path = WidgetPath{ widget: widget.clone(), children: Vec::new() };
	for child_arrangement in widget.get().arrange_children(geometry).iter().rev() {
		if !child_arrangement.geometry.contains_absolute_pos(pos) {
			continue
		}
		path.children.push(get_widget_path_under_position(child_arrangement.geometry, child_arrangement.widget.clone(), pos));
	}
	path
}

pub fn bubble_event(path: &WidgetPath, event: &WidgetEvent) -> Reply {
	for widget in path.bubble() {
		let reply = widget.get().on_event(event);
		if reply.handled {
			return reply
		}
	}
	Reply::unhandled()
}

pub struct EventContext {
	cursors: HashMap<usize, CursorEventContext>,
}

pub struct CursorEventContext {
	last_over_widgets: HashSet<WidgetRef<dyn Widget>>,
	captured_by_widget: Option<WidgetRef<dyn Widget>>,
}

impl EventContext {
	pub fn new() -> Self {
		EventContext {
			cursors: HashMap::new(),
		}
	}

	pub fn handle_mouse_move(&mut self, widget_path: &WidgetPath, cursor_index: usize, pos: &Vector2<scalar>) {
		let cursor_ctx = self.cursors.entry(cursor_index).or_insert(CursorEventContext{
			last_over_widgets: Default::default(),
			captured_by_widget: None,
		});

		let mut over_widgets: HashSet<WidgetRef<dyn Widget>> = Default::default();
		for widget in widget_path.bubble() {
			over_widgets.insert(widget.clone());
			if !cursor_ctx.last_over_widgets.remove(widget) {
				let event = WidgetEvent::OnMouseEnter;
				widget.get().on_event(&event);
			}
			let event = WidgetEvent::OnMouseMove;
			widget.get().on_event(&event);
		}

		for widget in &cursor_ctx.last_over_widgets {
			let event = WidgetEvent::OnMouseLeave;
			widget.get().on_event(&event);
		}
		cursor_ctx.last_over_widgets = over_widgets;
	}
}

