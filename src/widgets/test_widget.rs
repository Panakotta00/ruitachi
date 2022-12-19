use cgmath::Vector2;
use crate::paint::Painter;
use crate::util::{Geometry, WidgetRef};
use crate::widgets::{Widget, WidgetState};
use rand::Rng;
use skia_safe::{Paint, Rect, scalar};

pub struct TestWidget {
	widget: WidgetState,
	paint: Paint,
	size: Vector2<scalar>,
}

pub struct TestWidgetBuilder(TestWidget);

static mut TEST_WIDGET_RAND: Option<rand::rngs::ThreadRng> = None;

impl TestWidget {
	pub fn new() -> TestWidgetBuilder {
		let rng = if let Some(rng) = unsafe { &mut TEST_WIDGET_RAND } {
			rng
		} else {
			unsafe {
				TEST_WIDGET_RAND = Some(rand::thread_rng());
				TEST_WIDGET_RAND.as_mut().unwrap()
			}
		};

		let mut paint = Paint::default();
		paint.set_color(unsafe {
			skia_safe::HSV::from((rng.gen_range(0.0..360.0), 1.0, 1.0)).to_color(255)
		});
		TestWidgetBuilder(TestWidget {
			widget: WidgetState::default(),
			paint,
			size: Vector2::new(10.0, 10.0)
		})
	}
}

impl TestWidgetBuilder {
	pub fn size(mut self, size: Vector2<scalar>) -> Self {
		self.0.size = size;
		self
	}

	pub fn build(self) -> WidgetRef<TestWidget> {
		WidgetRef::new(self.0)
	}
}

impl Widget for TestWidget {
	fn widget_state(&self) -> &WidgetState {
		&self.widget
	}

	fn widget_state_mut(&mut self) -> &mut WidgetState {
		&mut self.widget
	}

	fn paint(&self, geometry: Geometry, layer: i32, painter: &mut Painter) -> i32 {
		let pos = geometry.local_pos();
		let size = geometry.local_size();
		painter.canvas().draw_rect(
			Rect::new(pos.x, pos.y, pos.x + size.x, pos.y + size.y),
			&self.paint,
		);
		layer + 1
	}

	fn get_desired_size(&self) -> Vector2<scalar> {
		self.size
	}
}
