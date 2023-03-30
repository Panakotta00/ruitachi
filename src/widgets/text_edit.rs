use std::cell::{Ref, RefMut};
use crate::{
	events::{Reply, WidgetEvent, WidgetFocusChange},
	paint::Painter,
	util::{Geometry, WidgetRef},
	widgets::{Widget, WidgetState},
};
use cgmath::Vector2;
use skia_safe::{scalar, Color, Font, Paint, Point, Rect};
use skia_safe::wrapper::ValueWrapper;
use winit::event::VirtualKeyCode;
use crate::widgets::{Arrangements, Children, WidgetImpl};
use crate::widgets::leaf_widget::{LeafState, LeafWidget};

pub struct TextEditWidgetState {
	leaf: LeafState,
	text: String,
	cursor: usize,
	cursor_byte: usize,
	foreground_font: Font,
	foreground: Paint,
	on_text_changed: Option<Box<dyn Fn(&str, &str)>>,
}

pub type TextEditWidget = WidgetImpl<TextEditWidgetState>;

pub struct TextEditWidgetBuilder(TextEditWidget);

impl TextEditWidget {
	pub fn new() -> TextEditWidgetBuilder {
		let mut paint = Paint::default();
		paint.set_color(Color::WHITE);
		let font = Font::default();
		TextEditWidgetBuilder(TextEditWidgetState {
			leaf: Default::default(),
			text: "".to_string(),
			cursor: 0,
			cursor_byte: 0,
			foreground_font: font,
			foreground: paint,
			on_text_changed: None,
		}.into())
	}

	pub fn set_cursor(&self, cursor: usize) {
		let mut state = self.state_mut();
		state.cursor = cursor.clamp(0, state.text.len());
		state.cursor_byte = state
			.text
			.char_indices()
			.map(|(i, _)| i)
			.nth(state.cursor)
			.unwrap_or(state.text.len());
	}

	pub fn set_text(&self, text: String) {
		let old_text = self.state().text.clone();
		if old_text != text {
			self.state().on_text_changed.as_ref().inspect(|d| d(&text, &old_text));
			self.state_mut().text = text;
			self.get_parent().inspect(|p| p.get().arrange_children(p.get().cached_geometry()));
		}
	}
}

impl TextEditWidgetBuilder {
	pub fn on_text_changed<F>(mut self, event: F) where F: Fn(&str, &str) + 'static {
		self.0.state_mut().on_text_changed = Some(Box::new(event));
	}

	pub fn build(self) -> WidgetRef<TextEditWidget> {
		WidgetRef::new(self.0)
	}
}

impl Widget for TextEditWidget {
	fn widget_state(&self) -> Ref<WidgetState> {
		self.widget_state(|v| &v.leaf.widget)
	}

	fn widget_state_mut(&self) -> RefMut<WidgetState> {
		self.widget_state_mut(|v| &mut v.leaf.widget)
	}

	fn get_desired_size(&self) -> Vector2<scalar> {
		let state = self.state();
		let text = if state.text.len() > 0 {
			state.text.as_str()
		} else {
			""
		};
		let text_size = state
			.foreground_font
			.measure_str(text, Some(&state.foreground));
		let m = state.foreground_font.metrics();
		Vector2::new(text_size.1.width(), m.1.bottom - m.1.top)
	}

	fn paint(&self, geometry: Geometry, layer: i32, painter: &mut Painter) -> i32 {
		let state = self.state();
		let width = if state.text.len() >= state.cursor && state.cursor > 0 {
			state.foreground_font
				.measure_str(&state.text[0..state.cursor_byte], Some(&state.foreground))
		} else {
			(0.0, Rect::default())
		};
		let center = geometry.local_size() / 2.0;
		let font_metric = state.foreground_font.metrics();
		let line_height = font_metric.1.bottom - font_metric.1.top;
		let base_line = center.y - line_height / 2.0 - font_metric.1.top;
		painter.draw_str(
			&state.text,
			Point::new(0.0, base_line),
			&state.foreground_font,
			&state.foreground,
		);
		painter.draw_line(
			Point::new(width.0, base_line + font_metric.1.top),
			Point::new(width.0, base_line + font_metric.1.bottom),
			&state.foreground,
		);
		layer + 1
	}

	fn on_event(&self, event: &WidgetEvent) -> Reply {
		let cursor_byte = self.state().cursor_byte;
		let cursor = self.state().cursor;
		let text = self.state().text.clone();
		match event {
			WidgetEvent::OnText {
				keyboard: _,
				character,
			} => {
				match *character {
					'\u{8}' => {
						if cursor > 0 {
							self.set_cursor(cursor - 1);
							self.state_mut().text.remove(cursor_byte);
						}
					}
					'\u{7F}' => {
						if cursor_byte < text.len() {
							self.state_mut().text.remove(cursor_byte);
						}
					}
					_ => {
						self.state_mut().text.insert(cursor_byte, *character);
						self.set_cursor(cursor + 1);
					}
				}
				self.get_parent().unwrap().get().arrange_children(self.get_parent().unwrap().get().cached_geometry());
				Reply::handled()
			}
			WidgetEvent::OnKeyDown {
				keyboard: _,
				key_physical: _,
				key: Some(key),
			} => match key {
				VirtualKeyCode::Left => {
					self.set_cursor(cursor.saturating_sub(1));
					Reply::handled()
				}
				VirtualKeyCode::Right => {
					self.set_cursor(cursor + 1);
					Reply::handled()
				}
				_ => Reply::unhandled(),
			},
			WidgetEvent::OnClick {
				mouse,
				button: _,
				pos: _,
			} => Reply::handled().take_focus(WidgetFocusChange::KeyboardList(vec![*mouse])),
			WidgetEvent::OnFocus { .. } => Reply::handled(),
			_ => Reply::unhandled(),
		}
	}

	fn get_children(&self) -> Children {
		self.leaf_get_children()
	}

	fn arrange_children(&self, geometry: Geometry) {
		self.leaf_arrange_children(geometry)
	}

	fn get_arranged_children(&self) -> Arrangements {
		self.leaf_get_arranged_children()
	}

	fn cached_geometry(&self) -> Geometry {
		self.cached_geometry()
	}
}

impl LeafWidget for TextEditWidget {
	fn leaf_state(&self) -> Ref<LeafState> {
		self.widget_state(|v| &v.leaf)
	}

	fn leaf_state_mut(&self) -> RefMut<LeafState> {
		self.widget_state_mut(|v| &mut v.leaf)
	}
}
