use crate::util::{Geometry, WidgetRef};
use crate::widgets::{Widget, WidgetState};
use cgmath::Vector2;
use rand::Rng;
use skia_safe::{scalar, Color, Paint, Rect};

pub trait Window {
	fn draw(&mut self, painter: &mut crate::paint::Painter);
}

pub struct WindowWidget {
	widget: WidgetState,
	content: Option<WidgetRef<dyn Widget>>,
}

pub struct WindowWidgetBuilder(WindowWidget);

impl WindowWidget {
	pub fn new(content: Option<WidgetRef<dyn Widget>>) -> WindowWidgetBuilder {
		WindowWidgetBuilder(WindowWidget {
			widget: WidgetState::default(),
			content,
		})
	}
}

impl WindowWidgetBuilder {
	pub fn build(self) -> WidgetRef<WindowWidget> {
		WidgetRef::new(self.0)
	}
}

impl Window for WindowWidget {
	fn draw(&mut self, painter: &mut crate::paint::Painter) {
		/*let mut rnd = rand::thread_rng();
		let canvas = painter.canvas();
		canvas.clear(skia_safe::Color::DARK_GRAY);
		let mut paint = skia_safe::Paint::default();
		paint.set_anti_alias(true);
		//paint.set_color(skia_safe::Color::new(rnd.gen_range(0..0xFFFFFF) << 8 | 0xFF));
		paint.set_color(skia_safe::Color::BLUE);
		canvas.draw_circle((100, 100), 90.0, &paint);*/

		let canvas = painter.canvas();
		canvas.clear(skia_safe::Color::DARK_GRAY);

		if let Some(content) = &self.content {
			let geometry = Geometry::new(
				Vector2::new(0.0, 0.0),
				Vector2::new(painter.width() as scalar, painter.height() as scalar),
				Vector2::new(0.0, 0.0),
				Vector2::new(1.0, 1.0),
			);
			content.get().paint(geometry, 0, painter);
		}

		/*let mut paint = Paint::default();
		paint.set_color(Color::BLUE);

		canvas.draw_rect(Rect::new(100.0, 100.0, 200.0, 200.0), &paint);*/
	}
}
