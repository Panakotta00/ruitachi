mod events;
pub mod input;

use crate::events::input::MouseButton;
use crate::platform::common::PlatformContext;
use crate::util::{Geometry, WidgetRef};
use crate::widgets::{Widget, Window};
use cgmath::Vector2;
pub use events::*;
use skia_safe::scalar;
use std::collections::{HashMap, HashSet};
use std::rc::{Rc, Weak};
use winit::event::VirtualKeyCode;

pub enum WidgetFocusChange {
	KeyboardList(Vec<usize>),
	AllKeyboards,
}

pub struct Reply {
	handled: bool,
	take_focus: Option<WidgetFocusChange>,
	free_focus: Option<WidgetFocusChange>,
	capture_cursor: Option<usize>,
	release_cursor: Option<usize>,
}

impl Reply {
	pub fn unhandled() -> Reply {
		Reply {
			handled: false,
			take_focus: None,
			free_focus: None,
			capture_cursor: None,
			release_cursor: None,
		}
	}

	pub fn handled() -> Reply {
		Reply {
			handled: true,
			take_focus: None,
			free_focus: None,
			capture_cursor: None,
			release_cursor: None,
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

	pub fn capture_cursor(mut self, cursor: usize) -> Self {
		self.capture_cursor = Some(cursor);
		self
	}

	pub fn release_cursor(mut self, cursor: usize) -> Self {
		self.release_cursor = Some(cursor);
		self
	}
}

pub enum WidgetEvent {
	OnCursorEnter {
		cursor: usize,
	},
	OnCursorMove {
		cursor: usize,
		pos: Vector2<scalar>,
	},
	OnCursorLeave {
		cursor: usize,
	},
	OnClick {
		mouse: usize,
		button: MouseButton,
		pos: Vector2<scalar>,
	},
	OnMouseButtonDown {
		mouse: usize,
		button: MouseButton,
		pos: Vector2<scalar>,
	},
	OnMouseButtonUp {
		mouse: usize,
		button: MouseButton,
		pos: Vector2<scalar>,
	},

	OnKeyDown {
		keyboard: usize,
		key_physical: usize,
		key: Option<VirtualKeyCode>,
	},
	OnKeyUp {
		keyboard: usize,
		key_physical: usize,
		key: Option<VirtualKeyCode>,
	},
	OnText {
		keyboard: usize,
		character: char,
	},
	OnFocus {
		keyboard: usize,
	},
	OnUnfocus {
		keyboard: usize,
	},
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
	iter: Box<WidgetPathIteratorBubbleInternal<'a>>,
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
				break;
			}

			// first or next children, so we need to wrap in new iterator for that child
			let mut next = Box::new(WidgetPathIteratorBubbleInternal {
				path: Some(&path.children[idx]),
				path_child_index: None,
				prev_iterator: None,
			});
			std::mem::swap(&mut self.iter, &mut next);
			self.iter.prev_iterator = Some(next);
		}

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

pub fn get_widget_path_under_position(
	geometry: Geometry,
	widget: WidgetRef<dyn Widget>,
	pos: &Vector2<scalar>,
) -> WidgetPath {
	let mut path = WidgetPath {
		widget: widget.clone(),
		children: Vec::new(),
	};
	for child_arrangement in widget
		.get()
		.calculate_arrange_children(geometry)
		.iter()
		.rev()
	{
		if !child_arrangement.geometry.contains_absolute_pos(pos) {
			continue;
		}
		path.children.push(get_widget_path_under_position(
			child_arrangement.geometry,
			child_arrangement.widget.clone(),
			pos,
		));
	}
	path
}

pub fn bubble_event(path: &WidgetPath, event: &WidgetEvent) -> Reply {
	for widget in path.bubble() {
		let reply = widget.get().on_event(event);
		if reply.handled {
			return reply;
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
	keyboard_index: usize,
	focused_widget: Option<WidgetRef<dyn Widget>>,
}

impl KeyboardEventContext {
	pub fn change_focus(&mut self, widget: Option<WidgetRef<dyn Widget>>) {
		if self.focused_widget == widget {
			return;
		}

		if let Some(already_focused) = &self.focused_widget {
			let unfocus_event = WidgetEvent::OnUnfocus {
				keyboard: self.keyboard_index,
			};
			already_focused.get().on_event(&unfocus_event);
		}
		if let Some(widget) = widget.as_ref() {
			let focus_event = WidgetEvent::OnFocus {
				keyboard: self.keyboard_index,
			};
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
		self.cursors
			.entry(cursor_index)
			.or_insert(CursorEventContext {
				last_over_widgets: HashSet::default(),
				captured_by_widget: None,
				about_to_be_clicked: HashMap::default(),
			})
	}

	pub fn try_get_cursor_context(
		&mut self,
		cursor_index: usize,
	) -> Option<&mut CursorEventContext> {
		self.cursors.get_mut(&cursor_index)
	}

	pub fn get_keyboard_context(&mut self, keyboard_index: usize) -> &mut KeyboardEventContext {
		self.keyboards
			.entry(keyboard_index)
			.or_insert(KeyboardEventContext {
				keyboard_index,
				focused_widget: None,
			})
	}

	pub fn try_get_keyboard_context(
		&mut self,
		keyboard_index: usize,
	) -> Option<&mut KeyboardEventContext> {
		self.keyboards.get_mut(&keyboard_index)
	}

	pub fn change_focus(&mut self, keyboard: usize, widget: Option<WidgetRef<dyn Widget>>) {
		let keyboard = self.get_keyboard_context(keyboard);
		keyboard.change_focus(widget);
	}

	pub fn process_reply(
		&mut self,
		widget: &WidgetRef<dyn Widget>,
		reply: &Reply,
		platform: &mut dyn PlatformContext,
	) {
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
				match change {
					WidgetFocusChange::KeyboardList(list) => {
						for keyboard in list {
							let ctx = self.get_keyboard_context(*keyboard);
							let has_focused = ctx.focused_widget.as_ref() == Some(widget);
							if has_focused {
								let unfocus_event = WidgetEvent::OnUnfocus {
									keyboard: *keyboard,
								};
								ctx.change_focus(None);
							}
						}
					}
					WidgetFocusChange::AllKeyboards => {
						for (_, ctx) in self.keyboards.iter_mut() {
							let has_focused = ctx.focused_widget.as_ref() == Some(widget);
							if has_focused {
								ctx.change_focus(None);
							}
						}
					}
				}
			}
			if let Some(cursor) = reply.capture_cursor {
				let cursor_ctx = self.get_cursor_context(cursor);
				let capture = &mut cursor_ctx.captured_by_widget;
				if capture.is_none() {
					*capture = Some(widget.clone());
					platform.set_capture_cursor(cursor, true);
				}
			}
			if let Some(cursor) = reply.release_cursor {
				let capture = &mut self.get_cursor_context(cursor).captured_by_widget;
				if let Some(captured_widget) = capture {
					if captured_widget == widget {
						*capture = None;
						platform.set_capture_cursor(cursor, false);
					}
				}
			}
		}
	}

	pub fn handle_mouse_move(
		&mut self,
		widget_path: &WidgetPath,
		cursor_index: usize,
		pos: &Vector2<scalar>,
	) {
		let cursor_ctx = self.get_cursor_context(cursor_index);

		let enter_event = WidgetEvent::OnCursorEnter {
			cursor: cursor_index,
		};
		let move_event = WidgetEvent::OnCursorMove {
			cursor: cursor_index,
			pos: *pos,
		};
		let leave_event = WidgetEvent::OnCursorLeave {
			cursor: cursor_index,
		};

		if let Some(captured_cursor) = &cursor_ctx.captured_by_widget {
			captured_cursor.get().on_event(&move_event);
		} else {
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
	}

	pub fn handle_mouse_button_down(
		&mut self,
		platform: &mut dyn PlatformContext,
		widget_path: &WidgetPath,
		mouse_index: usize,
		button: MouseButton,
		pos: &Vector2<scalar>,
	) {
		let cursor_ctx = self.get_cursor_context(mouse_index);

		let down_event = WidgetEvent::OnMouseButtonDown {
			mouse: mouse_index,
			button,
			pos: *pos,
		};

		if let Some(captured_cursor) = cursor_ctx.captured_by_widget.clone() {
			let reply = captured_cursor.get().on_event(&down_event);
			self.process_reply(&captured_cursor, &reply, platform);
		} else {
			let mut down_widgets: HashSet<WidgetRef<dyn Widget>> = HashSet::new();
			for widget in widget_path.bubble() {
				down_widgets.insert(widget.clone());
				let down_reply = widget.get().on_event(&down_event);
				self.process_reply(widget, &down_reply, platform);
				if down_reply.handled {
					break;
				}
			}
			if down_widgets.len() > 0 {
				let cursor_ctx = self.get_cursor_context(mouse_index);
				*cursor_ctx.about_to_be_clicked.entry(0).or_default() = down_widgets;
			}
		}
	}

	pub fn handle_mouse_button_up(
		&mut self,
		platform: &mut dyn PlatformContext,
		widget_path: &WidgetPath,
		cursor_index: usize,
		button: MouseButton,
		pos: &Vector2<scalar>,
	) {
		let up_event = WidgetEvent::OnMouseButtonUp {
			mouse: 0,
			button,
			pos: *pos,
		};
		let click_event = WidgetEvent::OnClick {
			mouse: cursor_index,
			button,
			pos: *pos,
		};

		let cursor_ctx = self.get_cursor_context(cursor_index);
		if let Some(captured_cursor) = cursor_ctx.captured_by_widget.clone() {
			let reply = captured_cursor.get().on_event(&up_event);
			self.process_reply(&captured_cursor, &reply, platform);
		} else {
			let mut handled_click = None;
			let mut up_widgets: HashSet<WidgetRef<dyn Widget>> = HashSet::new();
			for widget in widget_path.bubble() {
				up_widgets.insert(widget.clone());
				let up_reply = widget.get().on_event(&up_event);
				let cursor_ctx = self.get_cursor_context(cursor_index);
				let about_to_be_clicked = cursor_ctx.about_to_be_clicked.get(&cursor_index);
				let reply = about_to_be_clicked.and_then(|about_to| {
					if handled_click.is_none() || about_to.contains(widget) {
						Some(widget.get().on_event(&click_event))
					} else {
						None
					}
				});
				if let Some(click_reply) = reply {
					if click_reply.handled {
						handled_click = Some(widget.clone());
						self.process_reply(widget, &click_reply, platform);
					}
				}

				if up_reply.handled {
					break;
				}
			}
			let cursor_ctx = self.get_cursor_context(cursor_index);
			cursor_ctx.about_to_be_clicked.remove(&cursor_index);

			let keyboard_ctx = self.try_get_keyboard_context(0);
			if let Some(keyboard_ctx) = keyboard_ctx {
				if handled_click.is_none()
					|| (keyboard_ctx.focused_widget.is_some()
						&& handled_click.as_ref().unwrap()
							!= keyboard_ctx.focused_widget.as_ref().unwrap())
				{
					keyboard_ctx.change_focus(None)
				}
			}
		}
	}

	pub fn handle_key_down(
		&mut self,
		keyboard_index: usize,
		key_physical: usize,
		key: Option<VirtualKeyCode>,
	) {
		let keyboard_ctx = self.try_get_keyboard_context(keyboard_index);
		if let Some(keyboard_ctx) = keyboard_ctx {
			if let Some(focused_widget) = &keyboard_ctx.focused_widget {
				let key_down_event = WidgetEvent::OnKeyDown {
					keyboard: keyboard_index,
					key_physical,
					key,
				};
				focused_widget.get().on_event(&key_down_event);
			}
		}
	}

	pub fn handle_key_up(
		&mut self,
		keyboard_index: usize,
		key_physical: usize,
		key: Option<VirtualKeyCode>,
	) {
		let keyboard_ctx = self.try_get_keyboard_context(keyboard_index);
		if let Some(keyboard_ctx) = keyboard_ctx {
			if let Some(focused_widget) = &keyboard_ctx.focused_widget {
				let key_up_event = WidgetEvent::OnKeyUp {
					keyboard: keyboard_index,
					key_physical,
					key,
				};
				focused_widget.get().on_event(&key_up_event);
			}
		}
	}

	pub fn handle_text(&mut self, keyboard_index: usize, character: char) {
		let keyboard_ctx = self.try_get_keyboard_context(keyboard_index);
		if let Some(keyboard_ctx) = keyboard_ctx {
			if let Some(focused_widget) = &keyboard_ctx.focused_widget {
				let text_event = WidgetEvent::OnText {
					keyboard: keyboard_index,
					character,
				};
				focused_widget.get().on_event(&text_event);
			}
		}
	}

	pub fn handle_cursor_leave(&mut self, cursor_index: usize) {
		let cursor_ctx = self.try_get_cursor_context(cursor_index);
		if let Some(cursor_ctx) = cursor_ctx {
			let cursor_leave_event = WidgetEvent::OnCursorLeave {
				cursor: cursor_index,
			};
			for widget in &cursor_ctx.last_over_widgets {
				widget.get().on_event(&cursor_leave_event);
			}
		}
	}
}
