use crate::{
	paint::Painter,
	util::{Geometry, WidgetRef},
	widgets::{
		Axis, Axis::Vertical, PanelWidget, ScrollBarWidget,
		ScrollPanelScrollBarVisibility::Visible, Widget, WidgetArrangement, WidgetState,
	},
};
use cgmath::Vector2;
use skia_bindings::SkClipOp;
use skia_safe::Rect;

enum ScrollPanelScrollValue {
	ScrollBar(ScrollPanelScrollBars),
	Static(Vector2<f64>),
}

pub struct ScrollPanelScrollBars {
	right_visibility: ScrollPanelScrollBarVisibility,
	right: Option<WidgetRef<ScrollBarWidget>>,
	bottom_visibility: ScrollPanelScrollBarVisibility,
	bottom: Option<WidgetRef<ScrollBarWidget>>,
	left_visibility: ScrollPanelScrollBarVisibility,
	left: Option<WidgetRef<ScrollBarWidget>>,
	top_visibility: ScrollPanelScrollBarVisibility,
	top: Option<WidgetRef<ScrollBarWidget>>,
}

pub enum ScrollPanelScrollBarVisibility {
	Hidden,
	Visible,
	VisibleOnOverflow,
}

impl ScrollPanelScrollBars {
	pub fn new() -> Self {
		Self {
			right_visibility: ScrollPanelScrollBarVisibility::Visible,
			right: None,
			bottom_visibility: ScrollPanelScrollBarVisibility::Hidden,
			bottom: None,
			left_visibility: ScrollPanelScrollBarVisibility::Hidden,
			left: None,
			top_visibility: ScrollPanelScrollBarVisibility::Hidden,
			top: None,
		}
	}

	pub fn right(
		mut self,
		scroll_bar: Option<WidgetRef<ScrollBarWidget>>,
		visibility: ScrollPanelScrollBarVisibility,
	) -> Self {
		self.right_visibility = visibility;
		self.right = scroll_bar;
		self
	}

	pub fn bottom(
		mut self,
		scroll_bar: Option<WidgetRef<ScrollBarWidget>>,
		visibility: ScrollPanelScrollBarVisibility,
	) -> Self {
		self.bottom_visibility = visibility;
		self.bottom = scroll_bar;
		self
	}

	pub fn left(
		mut self,
		scroll_bar: Option<WidgetRef<ScrollBarWidget>>,
		visibility: ScrollPanelScrollBarVisibility,
	) -> Self {
		self.left_visibility = visibility;
		self.left = scroll_bar;
		self
	}

	pub fn top(
		mut self,
		scroll_bar: Option<WidgetRef<ScrollBarWidget>>,
		visibility: ScrollPanelScrollBarVisibility,
	) -> Self {
		self.top_visibility = visibility;
		self.top = scroll_bar;
		self
	}
}

pub struct ScrollPanel {
	widget: WidgetState,
	lock_to_direction: Option<Axis>,
	scroll_value: ScrollPanelScrollValue,
	content: Option<WidgetRef<dyn Widget>>,
}

impl ScrollPanel {
	pub fn new() -> ScrollPanelBuilder {
		ScrollPanelBuilder(Self {
			widget: Default::default(),
			lock_to_direction: Some(Vertical),
			scroll_value: ScrollPanelScrollValue::ScrollBar(
				ScrollPanelScrollBars::new().right(None, Visible),
			),
			content: None,
		})
	}
}

pub struct ScrollPanelBuilder(ScrollPanel);

impl ScrollPanelBuilder {
	pub fn build(self) -> WidgetRef<ScrollPanel> {
		WidgetRef::new(self.0)
	}

	/*pub fn lock_to_direction(mut self, direction: Option<Axis>) -> Self {
		self.0.lock_to_direction = direction;
	}

	pub fn direction(mut self, direction: Axis) -> Self {
		self.0.direction = direction;
		self.0.scroll_value = ScrollBar(match direction {
			Axis::Vertical => ScrollPanelScrollBars::new().left(None, Visible),
			Axis::Horizontal => ScrollPanelScrollBars::new().right(None, Visible),
		});
		self
	}

	pub fn scroll_bar(mut self, scroll_bar: WidgetRef<ScrollBarWidget>) -> Self {
		self.0.scroll_value = ScrollPanelScrollValue::ScrollBar(Some(scroll_bar));
		self
	}

	pub fn scroll_static(mut self, value: Vector2<f64>) -> Self {
		self.0.scroll_value = ScrollPanelScrollValue::Static(value.clamp(0.0, 1.0));
		self
	}

	pub fn content(mut self, content: WidgetRef<dyn Widget>) -> Self {
		self.0.content = Some(content);
		self
	}*/
}

impl Widget for ScrollPanel {
	fn widget_state(&self) -> &WidgetState {
		&self.widget
	}

	fn widget_state_mut(&mut self) -> &mut WidgetState {
		&mut self.widget
	}

	fn arrange_children(&self, _geometry: Geometry) -> Vec<WidgetArrangement> {
		let arranged = Vec::new();

		/*let value = match &self.scroll_value {
			ScrollBar(Some(scroll_bar)) => scroll_bar.get().value(),
			ScrollBar(None) => 0.0,
			ScrollPanelScrollValue::Static(v) => v,
		};

		let desired_size = if let Some(content) = &self.content {
			content.get().get_desired_size()
		} else {
			Vector2::new(0.0, 0.0)
		};

		let available_size = geometry.local_size()
			- if let ScrollBar(Some(scroll_bar)) = &self.scroll_value {
				let v = self
					.direction
					.get_vec_axis(scroll_bar.get().get_desired_size())
					.1;
				arranged.push(geometry.child_widget(scroll_bar));
				self.direction.create_vec(0.0, v)
			} else {
				Vector2::new(0.0, 0.0)
			};

		let mut hidden_size = self
			.direction
			.get_vec_axis(desired_size - available_size)
			.0
			.clamp(0.0, f32::MAX);

		if let ScrollBar(Some(scroll_bar)) = &self.scroll_value {
			arranged.push(geometry.child_widget(
				scroll_bar.clone(),
				self.direction.create_vec(-hidden_size * value, 0.0),
				desired_size,
			));
		}*/

		arranged
	}

	fn paint(&self, geometry: Geometry, layer: i32, painter: &mut Painter) -> i32 {
		painter.clip_rect(
			Rect::new(0.0, 0.0, geometry.local_size().x, geometry.local_size().y),
			Some(SkClipOp::Intersect),
			None,
		);
		PanelWidget::paint(self, geometry, layer, painter)
	}
}

impl PanelWidget for ScrollPanel {}
