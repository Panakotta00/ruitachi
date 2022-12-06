extern crate core;

use std::cmp::max;
use std::os::raw::c_void;
use cgmath::InnerSpace;
use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};
use skia_bindings::SkColorType::kBGRA_8888_SkColorType;
use skia_safe::{Color4f, colors};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use winit::window::{Theme, Window};

use ruitachi::*;
use ruitachi::events::MouseButtonEvent;
use ruitachi::paint::Painter;
use ruitachi::platform::common::PlatformContext;
use ruitachi::widgets::{Clickable, ClickState, Hoverable, HoverState, MouseInteract};

static mut hovering: bool = true;

mod meep;

fn main() {
    let mut event_loop = EventLoop::new();
    let mut window = WindowBuilder::new()
        .with_title("Hello World")
        .with_decorations(true)
        .with_transparent(true)
        .build(&event_loop)
        .unwrap();

    let mut platform_context = platform::wayland::Context::new(&mut window, &mut event_loop);
    platform_context.run(&mut window, &mut event_loop);
    //crate::meep::run();
}

struct Test {
    hover_state : HoverState,
    click_state : ClickState,
}

impl Clickable for Test{
    fn get_click_state(&mut self) -> &mut ClickState {
        &mut self.click_state
    }
}

impl Hoverable for Test {
    fn get_hover_state(&mut self) -> &mut HoverState {
        &mut self.hover_state
    }
}

impl MouseInteract for Test {
    fn on_mouse_button_down(&mut self, event: MouseButtonEvent) {
        println!("Meep");
        (self as &mut dyn Clickable).on_mouse_button_down(event);
    }
}

fn draw(window : &mut winit::window::Window, painter : &mut paint::Painter) {
    let canvas = painter.canvas();

    //canvas.clear(if window.theme() == winit::window::Theme::Dark { skia_safe::Color::DARK_GRAY } else { skia_safe::Color::WHITE });

    let mut paint = skia_safe::Paint::default();
    paint.set_anti_alias(true);
    paint.set_color4f(if unsafe { hovering } {
        colors::BLUE
    } else {
        colors::RED
    }, None);

    canvas.draw_circle((100, 100), 90.0, &paint);

    if let Some(text) = skia_safe::TextBlob::new("Hello World", &skia_safe::Font::default()) {
        canvas.draw_text_blob(text, (200, 100), &paint);
    }

    //canvas.draw_circle((pos.x, pos.y), 5.0, &paint);
}