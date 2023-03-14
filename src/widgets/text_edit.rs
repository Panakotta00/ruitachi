use crate::{
	events::{Reply, WidgetEvent, WidgetFocusChange},
	paint::Painter,
	util::{Geometry, WidgetRef},
	widgets::{Widget, WidgetState},
};
use cgmath::Vector2;
use skia_safe::{scalar, Color, Font, Paint, Point, Rect};
use winit::event::VirtualKeyCode;

pub struct TextEditWidget {
	widget: WidgetState,
	text: String,
	cursor: usize,
	cursor_byte: usize,
	foreground_font: Font,
	foreground: Paint,
}

pub struct TextEditWidgetBuilder(TextEditWidget);

impl TextEditWidget {
	pub fn new() -> TextEditWidgetBuilder {
		let mut paint = Paint::default();
		paint.set_color(Color::WHITE);
		let font = Font::default();
		TextEditWidgetBuilder(TextEditWidget {
			widget: WidgetState::default(),
			text: "".to_string(),
			cursor: 0,
			cursor_byte: 0,
			foreground_font: font,
			foreground: paint,
		})
	}

	pub fn set_cursor(&mut self, cursor: usize) {
		self.cursor = cursor.clamp(0, self.text.len());
		self.cursor_byte = self
			.text
			.char_indices()
			.map(|(i, _)| i)
			.nth(self.cursor)
			.unwrap_or(self.text.len());
	}
}

impl TextEditWidgetBuilder {
	pub fn build(self) -> WidgetRef<TextEditWidget> {
		WidgetRef::new(self.0)
	}
}

impl Widget for TextEditWidget {
	fn widget_state(&self) -> &WidgetState {
		&self.widget
	}

	fn widget_state_mut(&mut self) -> &mut WidgetState {
		&mut self.widget
	}

	fn get_desired_size(&self) -> Vector2<scalar> {
		let text = if self.text.len() > 0 {
			self.text.as_str()
		} else {
			""
		};
		let text_size = self
			.foreground_font
			.measure_str(text, Some(&self.foreground));
		let m = self.foreground_font.metrics();
		Vector2::new(text_size.1.width(), m.1.bottom - m.1.top)
	}

	fn paint(&self, geometry: Geometry, layer: i32, painter: &mut Painter) -> i32 {
		let width = if self.text.len() >= self.cursor && self.cursor > 0 {
			self.foreground_font
				.measure_str(&self.text[0..self.cursor_byte], Some(&self.foreground))
		} else {
			(0.0, Rect::default())
		};
		let center = geometry.local_size() / 2.0;
		let font_metric = self.foreground_font.metrics();
		let line_height = font_metric.1.bottom - font_metric.1.top;
		let base_line = center.y - line_height / 2.0 - font_metric.1.top;
		painter.draw_str(
			&self.text,
			Point::new(0.0, base_line),
			&self.foreground_font,
			&self.foreground,
		);
		painter.draw_line(
			Point::new(width.0, base_line + font_metric.1.top),
			Point::new(width.0, base_line + font_metric.1.bottom),
			&self.foreground,
		);
		layer + 1
	}

	fn on_event(&mut self, event: &WidgetEvent) -> Reply {
		match event {
			WidgetEvent::OnText {
				keyboard: _,
				character,
			} => {
				println!("{:?}", *character as usize);
				match *character {
					'\u{8}' => {
						if self.cursor > 0 {
							self.set_cursor(self.cursor - 1);
							self.text.remove(self.cursor_byte);
						}
					}
					'\u{7F}' => {
						if self.cursor_byte < self.text.len() {
							self.text.remove(self.cursor_byte);
						}
					}
					_ => {
						self.text.insert(self.cursor_byte, *character);
						self.set_cursor(self.cursor + 1);
					}
				}
				Reply::handled()
			}
			WidgetEvent::OnKeyDown {
				keyboard: _,
				key_physical: _,
				key: Some(key),
			} => match key {
				VirtualKeyCode::Left => {
					self.set_cursor(self.cursor.saturating_sub(1));
					Reply::handled()
				}
				VirtualKeyCode::Right => {
					self.set_cursor(self.cursor + 1);
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
}
