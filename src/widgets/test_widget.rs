use crate::events::{Reply, WidgetEvent, WidgetFocusChange};
use crate::paint::Painter;
use crate::util::{Geometry, WidgetRef};
use crate::widgets::{Widget, WidgetState};
use cgmath::Vector2;
use rand::Rng;
use skia_safe::{scalar, Paint, Rect};
use std::fmt::Debug;

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
			skia_safe::HSV::from((val, 1.0, 1.0)).to_color(self.paint.alpha())
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
		let size = geometry.local_size();
		painter.draw_rect(
			Rect::new(0.0, 0.0, size.x, size.y),
			&self.paint,
		);
		layer + 1
	}

	fn get_desired_size(&self) -> Vector2<scalar> {
		self.size
	}

	fn on_event(&mut self, event: &WidgetEvent) -> Reply {
		match event {
			WidgetEvent::OnCursorEnter { cursor } => {
				println!("Mouse {} Enter for {}", cursor, self.name);
				self.paint.set_alpha(150);
			}
			WidgetEvent::OnCursorMove { .. } => {
				//println!("Mouse Move for {} {}!!!", self.name, self.counter);
				//self.counter += 1;
			}
			WidgetEvent::OnCursorLeave { cursor } => {
				println!("Mouse {} Leave for {}", cursor, self.name);
				self.paint.set_alpha(255);
			}
			WidgetEvent::OnClick { mouse, pos, button } => {
				println!(
					"Mouse {} Click {:?} for {} '{}' at {:?}!!!",
					mouse, button, self.name, self.counter, pos
				);
				self.counter += 1;
				self.random_color();
				return Reply::handled().take_focus(WidgetFocusChange::KeyboardList(vec![0]));
			}
			WidgetEvent::OnMouseButtonDown { .. } => {}
			WidgetEvent::OnMouseButtonUp { .. } => {}
			WidgetEvent::OnKeyDown {
				keyboard,
				key_physical,
				key,
			} => {
				println!(
					"Key '{}' down for {} from {}!",
					key_physical, self.name, keyboard
				);
			}
			WidgetEvent::OnKeyUp {
				keyboard,
				key_physical,
				key,
			} => {
				println!(
					"Key '{}' up for {} from {}!",
					key_physical, self.name, keyboard
				);
			}
			WidgetEvent::OnText {
				keyboard,
				character,
			} => {
				println!("Text '{}' for {} from {}!", character, self.name, keyboard);
			}
			WidgetEvent::OnFocus { keyboard } => {
				println!("Focused {} from {}!", self.name, keyboard);
			}
			WidgetEvent::OnUnfocus { keyboard } => {
				println!("Unfocused {} from {}!", self.name, keyboard);
			}
		}
		Reply::handled()
	}
}
