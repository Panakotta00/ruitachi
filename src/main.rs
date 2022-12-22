#![feature(coerce_unsized)]
#![feature(unsize)]
#![feature(trait_upcasting)]

extern crate core;

use skia_safe::colors;
use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::rc::Rc;
use cgmath::Vector2;
use winit::{event_loop::EventLoop, window::WindowBuilder};

use ruitachi::events::MouseButtonEvent;
use ruitachi::*;

use ruitachi::platform::common::PlatformContext;
use ruitachi::util::WidgetRef;
use ruitachi::widgets::{BoxPanel, Growth, HorizontalAlignment, LinearPanelDirection, LinearPanel, OverlayPanel, TestWidget, VerticalAlignment, WindowWidget};

fn main() {
	let mut event_loop = EventLoop::new();
	let mut window = WindowBuilder::new()
		.with_title("Hello World")
		.with_decorations(true)
		.with_transparent(true)
		.build(&event_loop)
		.unwrap();

	let test1 = TestWidget::new().build();
	let test2 = TestWidget::new().name("Test2").build();
	let test21 = TestWidget::new().size(Vector2::new(20.0, 20.0)).name("Test21").build();

	let test3 = TestWidget::new().build();
	let test4 = TestWidget::new().build();

	let test5 = TestWidget::new().build();
	let test6 = TestWidget::new().build();
	let test7 = TestWidget::new().build();
	let panel = LinearPanel::new(LinearPanelDirection::Horizontal)
		.slot(test5, Growth::Fill)
		.slot(test6, Growth::Fit)
		.slot(test7,Growth::Fit)
		.build();

	let panel = LinearPanel::new(LinearPanelDirection::Vertical)
		.slot(test3, Growth::Val(1.0))
		.slot(test4, Growth::Val(0.1))
		.slot(panel, Growth::Val(1.0))
		.build();

	let panel = LinearPanel::new(LinearPanelDirection::Horizontal)
		.slot(
			BoxPanel::new(test1)
				.v_align(VerticalAlignment::Center)
				.h_align(HorizontalAlignment::Center)
				.override_y(20.0)
				.build(), Growth::Fill)
		.slot(
			OverlayPanel::new()
				.slot(BoxPanel::new(test21).override_size(Vector2::new(200.0, 200.0)).build())
				.slot(BoxPanel::new(test2).override_size(Vector2::new(100.0, 100.0)).build())
				.build(), Growth::Fit)
		.slot(panel, Growth::Fill)
		.build();

	let mut window_widget = WindowWidget::new(Some(panel)).build();

	let mut platform_context = platform::Context::new(&mut window, &mut event_loop, window_widget);

	platform_context.run(&mut window, &mut event_loop);
}
