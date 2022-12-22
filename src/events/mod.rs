mod events;

use std::collections::{HashMap, HashSet};
use std::rc::{Rc, Weak};
use cgmath::Vector2;
use skia_safe::scalar;
pub use events::*;
use crate::util::{Geometry, WidgetRef};
use crate::widgets::{Widget, Window};

pub enum WidgetFocusChange {
	KeyboardList(Vec<usize>),
	AllKeyboards,
}

pub struct Reply {
	handled: bool,
	take_focus: Option<WidgetFocusChange>,
	free_focus: Option<WidgetFocusChange>,
}

impl Reply {
	pub fn unhandled() -> Reply {
		Reply {
			handled: false,
			take_focus: None,
			free_focus: None,
		}
	}

	pub fn handled() -> Reply {
		Reply {
			handled: true,
			take_focus: None,
			free_focus: None,
		}
	}

	pub fn take_focus(mut self, change: WidgetFocusChange) -> Self {
		self.take_focus = Some(change);
		self
	}

	pub fn free_focus(mut self, change: WidgetFocusChange) -> Self {
		self.free_focus = Some(change);
		self
	}
}

pub enum WidgetEvent {
	OnCursorEnter,
	OnCursorMove,
	OnCursorLeave,
	OnMouseButtonDown,
	OnMouseButtonUp,
	OnClick,

	OnKeyDown,
	OnKeyUp,
	OnText,
	OnFocus,
	OnUnfocus,
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
	keyboards: HashMap<usize, KeyboardEventContext>,
}

pub struct CursorEventContext {
	last_over_widgets: HashSet<WidgetRef<dyn Widget>>,
	captured_by_widget: Option<WidgetRef<dyn Widget>>,
	about_to_be_clicked: HashMap<usize, HashSet<WidgetRef<dyn Widget>>>,
}

pub struct KeyboardEventContext {
	focused_widget: Option<WidgetRef<dyn Widget>>,
}

impl KeyboardEventContext {
	pub fn change_focus(&mut self, widget: Option<WidgetRef<dyn Widget>>) {
		if let Some(already_focused) = &self.focused_widget {
			let unfocus_event = WidgetEvent::OnUnfocus;
			already_focused.get().on_event(&unfocus_event);
		}
		if let Some(widget) = &widget {
			let focus_event = WidgetEvent::OnFocus;
			widget.get().on_event(&focus_event);
		}
		self.focused_widget = widget;
	}
}

impl EventContext {
	pub fn new() -> Self {
		EventContext {
			cursors: HashMap::new(),
			keyboards: HashMap::new(),
		}
	}

	pub fn get_cursor_context(&mut self, cursor_index: usize) -> &mut CursorEventContext {
		self.cursors.entry(cursor_index).or_insert(CursorEventContext{
			last_over_widgets: HashSet::default(),
			captured_by_widget: None,
			about_to_be_clicked: HashMap::default(),
		})
	}

	pub fn get_keyboard_context(&mut self, keyboard_index: usize) -> &mut KeyboardEventContext {
		self.keyboards.entry(keyboard_index).or_insert(KeyboardEventContext{
			focused_widget: None,
		})
	}

	pub fn try_get_keyboard_context(&mut self, keyboard_index: usize) -> Option<&mut KeyboardEventContext> {
		self.keyboards.get_mut(&keyboard_index)
	}

	pub fn change_focus(&mut self, keyboard: usize, widget: Option<WidgetRef<dyn Widget>>) {
		let keyboard = self.get_keyboard_context(keyboard);
		keyboard.change_focus(widget);
	}

	pub fn process_reply(&mut self, widget: &WidgetRef<dyn Widget>, reply: &Reply) {
		if reply.handled {
			if let Some(change) = &reply.take_focus {
				match change {
					WidgetFocusChange::KeyboardList(list) => {
						for keyboard in list {
							self.change_focus(*keyboard, Some(widget.clone()));
						}
					}
					WidgetFocusChange::AllKeyboards => {
						// TODO: User list of available keyboards, instead of currently registered ones
						for (_, ctx) in self.keyboards.iter_mut() {
							ctx.change_focus(Some(widget.clone()));
						}
					}
				}
			} else if let Some(change) = &reply.free_focus {
				let unfocus_event = WidgetEvent::OnUnfocus;
				match change {
					WidgetFocusChange::KeyboardList(list) => {
						for keyboard in list {
							let ctx = self.get_keyboard_context(*keyboard);
							let has_focused = ctx.focused_widget.as_ref() == Some(widget);
							if has_focused {
								ctx.focused_widget.as_ref().unwrap().get().on_event(&unfocus_event);
								ctx.focused_widget = None;
							}
						}
					}
					WidgetFocusChange::AllKeyboards => {
						let unfocus_event = WidgetEvent::OnUnfocus;

						for (_, ctx) in self.keyboards.iter_mut() {
							let has_focused = ctx.focused_widget.as_ref() == Some(widget);
							if has_focused {
								ctx.focused_widget.as_ref().unwrap().get().on_event(&unfocus_event);
								ctx.focused_widget = None;
							}
						}
					}
				}
			}
		}
	}

	pub fn handle_mouse_move(&mut self, widget_path: &WidgetPath, cursor_index: usize, pos: &Vector2<scalar>) {
		let cursor_ctx = self.get_cursor_context(cursor_index);

		let enter_event = WidgetEvent::OnCursorEnter;
		let move_event = WidgetEvent::OnCursorMove;
		let leave_event = WidgetEvent::OnCursorLeave;

		let mut over_widgets: HashSet<WidgetRef<dyn Widget>> = HashSet::new();
		for widget in widget_path.bubble() {
			over_widgets.insert(widget.clone());
			if !cursor_ctx.last_over_widgets.remove(widget) {
				widget.get().on_event(&enter_event);
			}
			widget.get().on_event(&move_event);
		}

		for widget in &cursor_ctx.last_over_widgets {
			widget.get().on_event(&leave_event);
		}
		cursor_ctx.last_over_widgets = over_widgets;
	}

	pub fn handle_mouse_button_down(&mut self, widget_path: &WidgetPath, cursor_index: usize, pos: &Vector2<scalar>) {
		let cursor_ctx = self.get_cursor_context(cursor_index);

		let down_event = WidgetEvent::OnMouseButtonDown;

		let mut down_widgets: HashSet<WidgetRef<dyn Widget>> = HashSet::new();
		for widget in widget_path.bubble() {
			down_widgets.insert(widget.clone());
			let down_reply = widget.get().on_event(&down_event);
			if down_reply.handled {
				break
			}
		}
		if down_widgets.len() > 0 {
			*cursor_ctx.about_to_be_clicked.entry(0).or_default() = down_widgets;
		}
	}

	pub fn handle_mouse_button_up(&mut self, widget_path: &WidgetPath, cursor_index: usize, pos: &Vector2<scalar>) {

		let up_event = WidgetEvent::OnMouseButtonUp;
		let click_event = WidgetEvent::OnClick;

		let mut handled_click = None;
		let mut up_widgets: HashSet<WidgetRef<dyn Widget>> = HashSet::new();
		for widget in widget_path.bubble() {
			up_widgets.insert(widget.clone());
			let up_reply = widget.get().on_event(&up_event);

			let cursor_ctx = self.get_cursor_context(cursor_index);
			let about_to_be_clicked = cursor_ctx.about_to_be_clicked.get(&0);
			let reply = about_to_be_clicked.and_then(|about_to| {
				if handled_click.is_none() || about_to.contains(widget) {
					Some(widget.get().on_event(&click_event))
				} else {
					None
				}
			});
			if let Some(click_reply) = reply {
				handled_click = Some(widget.clone());
				self.process_reply(widget, &click_reply);
			}

			if up_reply.handled {
				break
			}
		}
		let cursor_ctx = self.get_cursor_context(cursor_index);
		cursor_ctx.about_to_be_clicked.remove(&0);

		let keyboard_ctx = self.try_get_keyboard_context(0);
		if let Some(keyboard_ctx) = keyboard_ctx {
			if handled_click.is_none() || handled_click != keyboard_ctx.focused_widget {
				keyboard_ctx.change_focus(None)
			}
		}
	}

	pub fn handle_key_down(&mut self, keyboard_index: usize) {
		let keyboard_ctx = self.try_get_keyboard_context(keyboard_index);
		if let Some(keyboard_ctx) = keyboard_ctx {
			if let Some(focused_widget) = &keyboard_ctx.focused_widget {
				let key_down_event = WidgetEvent::OnKeyDown;
				focused_widget.get().on_event(&key_down_event);
			}
		}
	}

	pub fn handle_key_up(&mut self, keyboard_index: usize) {
		let keyboard_ctx = self.try_get_keyboard_context(keyboard_index);
		if let Some(keyboard_ctx) = keyboard_ctx {
			if let Some(focused_widget) = &keyboard_ctx.focused_widget {
				let key_up_event = WidgetEvent::OnKeyUp;
				focused_widget.get().on_event(&key_up_event);
			}
		}
	}

	pub fn handle_text(&mut self, keyboard_index: usize) {
		let keyboard_ctx = self.try_get_keyboard_context(keyboard_index);
		if let Some(keyboard_ctx) = keyboard_ctx {
			if let Some(focused_widget) = &keyboard_ctx.focused_widget {
				let text_event = WidgetEvent::OnText;
				focused_widget.get().on_event(&text_event);
			}
		}
	}
}

