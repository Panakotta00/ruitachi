use cgmath::Vector2;
use crate::paint::Painter;
use crate::util::{Geometry, WidgetRef};
use crate::widgets::{Widget, WidgetState};
use rand::Rng;
use skia_safe::{Paint, Rect, scalar};
use crate::events::{Reply, WidgetEvent};

pub struct TestWidget {
	widget: WidgetState,
	paint: Paint,
	size: Vector2<scalar>,
	name: String,
	counter: i32,
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
			let val = rng.gen::<f32>() * 360.0;
			skia_safe::HSV::from((val, 1.0, 1.0)).to_color(255)
		});
		TestWidgetBuilder(TestWidget {
			widget: WidgetState::default(),
			paint,
			size: Vector2::new(10.0, 10.0),
			name: "Unnamed".into(),
			counter: 0,
		})
	}

	pub fn random_color(&mut self) {
		self.paint.set_color(unsafe {
			let val = TEST_WIDGET_RAND.as_mut().unwrap().gen::<f32>() * 360.0;
			skia_safe::HSV::from((val, 1.0, 1.0)).to_color(255)
		});
	}
}

impl TestWidgetBuilder {
	pub fn size(mut self, size: Vector2<scalar>) -> Self {
		self.0.size = size;
		self
	}

	pub fn name(mut self, name: &str) -> Self {
		self.0.name = name.into();
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

	fn on_event(&mut self, event: &WidgetEvent) -> Reply {
		match event {
			WidgetEvent::OnMouseEnter => {
				println!("Mouse Enter for {}", self.name);
				self.paint.set_alpha(150);
			}
			WidgetEvent::OnMouseMove => {
				//println!("Mouse Move for {} {}!!!", self.name, self.counter);
				//self.counter += 1;
			}
			WidgetEvent::OnMouseLeave => {
				println!("Mouse Leave for {}", self.name);
				self.paint.set_alpha(255);
			}
			WidgetEvent::OnMouseInput => {
				println!("Mouse Click for {} {}!!!", self.name, self.counter);
				self.counter += 1;
				self.random_color();
			}
		}
		Reply::handled()
	}
}
