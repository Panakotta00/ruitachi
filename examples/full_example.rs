#![feature(coerce_unsized)]
#![feature(unsize)]
#![feature(trait_upcasting)]

extern crate core;

use cgmath::Vector2;
use ruitachi::application::{GUIApplication};

use ruitachi::{
	platform::common::PlatformContext,
	util::WidgetRef,
	widgets::{
		Axis, BoxPanel, Growth, HorizontalAlignment, LinearPanel, LinearPanelDirection,
		OverlayPanel, ScrollBarWidget, ScrollPanel, ScrollPanelDirection, TestWidget,
		TextBlockWidget, TextEditWidget, VerticalAlignment, Window, WindowWidget,
	},
};
use ruitachi::util::SharedRef;

fn main() {
	let test1 = TestWidget::new().on_click(move || {
		let window: WidgetRef<dyn Window> = WindowWidget::new(None).build();
		GUIApplication::get().add_window(window);
	}).build();
	let test2 = TestWidget::new().name("Test2").build();
	let test21 = TestWidget::new()
		.size(Vector2::new(20.0, 20.0))
		.name("Test21")
		.build();

	let test3 = TestWidget::new().build();
	let test4 = TestWidget::new().build();

	let test5 = TestWidget::new().build();
	let test6 = TestWidget::new().build();
	let test7 = TestWidget::new().build();
	let panel = LinearPanel::new(LinearPanelDirection::Horizontal)
		.slot(test5, Growth::Fill)
		.slot(test6, Growth::Fit)
		.slot(test7, Growth::Fit)
		.build();

	let panel = LinearPanel::new(LinearPanelDirection::Vertical)
		.slot(test3, Growth::Val(1.0))
		.slot(test4, Growth::Val(0.1))
		.slot(panel, Growth::Val(1.0))
		.build();

	let panel = LinearPanel::new(LinearPanelDirection::Vertical)
		.slot(
			LinearPanel::new(LinearPanelDirection::Horizontal)
				.slot(
					OverlayPanel::new()
						.slot(
							BoxPanel::new(test1)
								.v_align(VerticalAlignment::Center)
								.h_align(HorizontalAlignment::Center)
								.override_y(20.0)
								.build(),
						)
						.slot(
							BoxPanel::new(
								ScrollPanel::new()
									.direction(ScrollPanelDirection::Horizontal)
									.content(TextEditWidget::new().build())
									.build(),
							)
							.v_align(VerticalAlignment::Fill)
							.h_align(HorizontalAlignment::Fill)
							.build(),
						)
						.build(),
					Growth::Fill,
				)
				.slot(
					OverlayPanel::new()
						.slot(
							BoxPanel::new(test21)
								.override_size(Vector2::new(200.0, 200.0))
								.build(),
						)
						.slot(
							BoxPanel::new(test2)
								.override_size(Vector2::new(100.0, 100.0))
								.build(),
						)
						.build(),
					Growth::Fit,
				)
				.slot(panel, Growth::Fill)
				.build(),
			Growth::Fill,
		)
		.slot(
			ScrollBarWidget::new().direction(Axis::Horizontal).build(),
			Growth::Fit,
		)
		.build();

	let window_widget1: WidgetRef<dyn Window> = WindowWidget::new(Some(panel)).build();

	let window2_box = LinearPanel::new(LinearPanelDirection::Horizontal)
		.slot(
			ScrollPanel::new()
				.direction(ScrollPanelDirection::Horizontal)
				.content(
					LinearPanel::new(LinearPanelDirection::Vertical)
						.slot(
							TestWidget::new().size(Vector2::new(300.0, 300.0)).build(),
							Growth::Fit,
						)
						.slot(TextEditWidget::new().build(), Growth::Fit)
						.slot(
							LinearPanel::new(LinearPanelDirection::Horizontal)
								.slot(TestWidget::new().build(), Growth::Fit)
								.slot(TestWidget::new().build(), Growth::Fit)
								.slot(TestWidget::new().build(), Growth::Fit)
								.slot(TestWidget::new().build(), Growth::Fit)
								.slot(TestWidget::new().build(), Growth::Fit)
								.slot(TestWidget::new().build(), Growth::Fit)
								.slot(TestWidget::new().build(), Growth::Fit)
								.slot(TestWidget::new().build(), Growth::Fit)
								.slot(TestWidget::new().build(), Growth::Fit)
								.slot(TestWidget::new().build(), Growth::Fit)
								.slot(TestWidget::new().build(), Growth::Fit)
								.slot(TestWidget::new().build(), Growth::Fit)
								.slot(TestWidget::new().build(), Growth::Fit)
								.slot(TestWidget::new().build(), Growth::Fit)
								.slot(TestWidget::new().build(), Growth::Fit)
								.build(),
							Growth::Fit,
						)
						.build(),
				)
				.build(),
			Growth::Fill,
		)
		.slot(TestWidget::new().build(), Growth::Fill)
		.build();
	let window_widget2: WidgetRef<dyn Window> = WindowWidget::new(Some(window2_box)).build();

	GUIApplication::get().add_window(window_widget1);
	GUIApplication::get().add_window(window_widget2);

	GUIApplication::get().run();
}
