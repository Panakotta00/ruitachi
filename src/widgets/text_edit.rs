use crate::events::{Reply, WidgetEvent, WidgetFocusChange};
use crate::paint::Painter;
use crate::util::{Geometry, WidgetRef};
use crate::widgets::{Widget, WidgetState};
use cgmath::Vector2;
use skia_safe::{scalar, Color, Font, Paint, Point, Rect};

pub struct TextEditWidget {
	widget: WidgetState,
	text: String,
	cursor: usize,
	foreground_font: Font,
	foreground: Paint,
}

pub struct TextEditWidgetBuilder(TextEditWidget);

impl TextEditWidget {
	pub fn new() -> TextEditWidgetBuilder {
		let mut paint = Paint::default();
		paint.set_color(Color::WHITE);
		let mut font = Font::default();
		TextEditWidgetBuilder(TextEditWidget {
			widget: WidgetState::default(),
			text: "".to_string(),
			cursor: 0,
			foreground_font: font,
			foreground: paint,
		})
	}
}

impl TextEditWidgetBuilder {
	pub fn build(mut self) -> WidgetRef<TextEditWidget> {
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
			"#"
		};
		let text_size = self
			.foreground_font
			.measure_str(text, Some(&self.foreground));
		Vector2::new(text_size.1.width(), text_size.1.height())
	}

	fn paint(&self, geometry: Geometry, layer: i32, painter: &mut Painter) -> i32 {
		let width = if self.text.len() >= self.cursor && self.cursor > 0 {
			self.foreground_font
				.measure_str(&self.text[0..self.cursor], Some(&self.foreground))
		} else {
			(0.0, Rect::default())
		};
		let Vector2 { x, y } = geometry.absolute_pos();
		painter.canvas().draw_str(
			&self.text,
			Point::new(x + 10.0, y + geometry.local_size().y / 2.0),
			&self.foreground_font,
			&self.foreground,
		);
		painter.canvas().draw_line(
			Point::new(x + width.0 + 10.0, y + 0.0),
			Point::new(x + width.0 + 10.0, y + geometry.local_size().y),
			&self.foreground,
		);
		layer + 1
	}

	fn on_event(&mut self, event: &WidgetEvent) -> Reply {
		match event {
			WidgetEvent::OnText {
				keyboard,
				character,
			} => {
				self.text.insert(self.cursor, *character);
				self.cursor += character.len_utf8();
				Reply::handled()
			}
			WidgetEvent::OnClick { mouse, button, pos } => {
				Reply::handled().take_focus(WidgetFocusChange::KeyboardList(vec![*mouse]))
			}
			WidgetEvent::OnFocus { .. } => Reply::handled(),
			_ => Reply::unhandled(),
		}
	}
}
