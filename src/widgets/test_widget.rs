use std::cell::{Ref, RefMut};
use crate::{
	events::{Reply, WidgetEvent, WidgetFocusChange},
	paint::Painter,
	util::{Geometry, WidgetRef},
	widgets::{Widget, WidgetState},
};
use cgmath::Vector2;
use rand::Rng;
use skia_safe::{scalar, Paint, Rect};
use crate::widgets::{Arrangements, Children, WidgetImpl};
use crate::widgets::leaf_widget::{LeafState, LeafWidget};

pub struct TestWidgetState {
	leaf: LeafState,
	paint: Paint,
	size: Vector2<scalar>,
	name: String,
	counter: i32,
}

pub type TestWidget = WidgetImpl<TestWidgetState>;

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
		TestWidgetBuilder(TestWidgetState {
			leaf: Default::default(),
			paint,
			size: Vector2::new(10.0, 10.0),
			name: "Unnamed".into(),
			counter: 0,
		}.into())
	}

	pub fn random_color(&mut self) {
		let mut state = self.state_mut();
		let alpha = state.paint.alpha();
		state.paint.set_color(unsafe {
			let val = TEST_WIDGET_RAND.as_mut().unwrap().gen::<f32>() * 360.0;
			skia_safe::HSV::from((val, 1.0, 1.0)).to_color(alpha)
		});
	}
}

impl TestWidgetBuilder {
	pub fn size(mut self, size: Vector2<scalar>) -> Self {
		self.0.state_mut().size = size;
		self
	}

	pub fn name(mut self, name: &str) -> Self {
		self.0.state_mut().name = name.into();
		self
	}

	pub fn build(self) -> WidgetRef<TestWidget> {
		WidgetRef::new(self.0)
	}
}

impl Widget for TestWidget {
	fn widget_state(&self) -> Ref<WidgetState> {
		self.widget_state(|v| &v.leaf.widget)
	}

	fn widget_state_mut(&mut self) -> RefMut<WidgetState> {
		self.widget_state_mut(|v| &mut v.leaf.widget)
	}

	fn paint(&self, geometry: Geometry, layer: i32, painter: &mut Painter) -> i32 {
		let size = geometry.local_size();
		painter.draw_rect(Rect::new(0.0, 0.0, size.x, size.y), &self.state().paint);
		layer + 1
	}

	fn get_desired_size(&self) -> Vector2<scalar> {
		self.state().size
	}

	fn get_children(&self) -> Children {
		self.leaf_get_children()
	}

	fn arrange_children(&mut self, geometry: Geometry) {
		self.leaf_arrange_children(geometry)
	}

	fn get_arranged_children(&self) -> Arrangements {
		self.leaf_get_arranged_children()
	}

	fn on_event(&mut self, event: &WidgetEvent) -> Reply {
		match event {
			WidgetEvent::OnCursorEnter { cursor } => {
				println!("Mouse {} Enter for {}", cursor, self.state().name);
				self.state_mut().paint.set_alpha(150);
			}
			WidgetEvent::OnCursorMove { .. } => {
				//println!("Mouse Move for {} {}!!!", self.name, self.counter);
				//self.counter += 1;
			}
			WidgetEvent::OnCursorLeave { cursor } => {
				println!("Mouse {} Leave for {}", cursor, self.state().name);
				self.state_mut().paint.set_alpha(255);
			}
			WidgetEvent::OnClick { mouse, pos, button } => {
				println!(
					"Mouse {} Click {:?} for {} '{}' at {:?}!!!",
					mouse, button, self.state().name, self.state().counter, pos
				);
				self.state_mut().counter += 1;
				self.random_color();
				return Reply::handled().take_focus(WidgetFocusChange::KeyboardList(vec![0]));
			}
			WidgetEvent::OnMouseButtonDown { .. } => {}
			WidgetEvent::OnMouseButtonUp { .. } => {}
			WidgetEvent::OnKeyDown {
				keyboard,
				key_physical,
				key: _,
			} => {
				println!(
					"Key '{}' down for {} from {}!",
					key_physical, self.state().name, keyboard
				);
			}
			WidgetEvent::OnKeyUp {
				keyboard,
				key_physical,
				key: _,
			} => {
				println!(
					"Key '{}' up for {} from {}!",
					key_physical, self.state().name, keyboard
				);
			}
			WidgetEvent::OnText {
				keyboard,
				character,
			} => {
				println!("Text '{}' for {} from {}!", character, self.state().name, keyboard);
			}
			WidgetEvent::OnFocus { keyboard } => {
				println!("Focused {} from {}!", self.state().name, keyboard);
			}
			WidgetEvent::OnUnfocus { keyboard } => {
				println!("Unfocused {} from {}!", self.state().name, keyboard);
			}
		}
		Reply::handled()
	}

	fn cached_geometry(&self) -> Geometry {
		self.leaf_cached_geometry()
	}
}

impl LeafWidget for TestWidget {
	fn leaf_state(&self) -> Ref<LeafState> {
		self.widget_state(|v| &v.leaf)
	}

	fn leaf_state_mut(&mut self) -> RefMut<LeafState> {
		self.widget_state_mut(|v| &mut v.leaf)
	}
}
