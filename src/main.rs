extern crate core;

use std::cmp::max;
use std::os::raw::c_void;
use cgmath::InnerSpace;
use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};
use skia_bindings::SkColorType::kBGRA_8888_SkColorType;
use skia_safe::{Color4f, colors};
use windows::Win32::Foundation::S_FALSE;
use windows::Win32::Graphics::Gdi::{BeginPaint, BI_RGB, BITMAPINFO, BITMAPINFOHEADER, DIB_RGB_COLORS, PAINTSTRUCT, RGBQUAD, SRCCOPY};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use winit::platform::windows::{WindowBuilderExtWindows, WindowExtWindows};
use winit::window::Theme;

use ruitachi::*;
use ruitachi::events::MouseButtonEvent;
use ruitachi::platform::common::Platform;
use ruitachi::widgets::{Clickable, ClickState, Hoverable, HoverState, MouseInteract};

fn main() {
    let mut hovering = false;
    let mut pos = cgmath::Vector2::<f32>::new(0.0, 0.0);
    let event_loop = EventLoop::new();
    let mut window = WindowBuilder::new()
        .with_title("Hello World")
        .with_decorations(true)
        .with_transparent(true)
        .build(&event_loop)
        .unwrap();

    platform::Platform::IntializeWindow(&mut window);

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id,
            } if window_id == window.id() => *control_flow = ControlFlow::Exit,
            Event::RedrawRequested(_) => {
                platform::Platform::DrawWindow(&mut window, draw)
            },
            Event::WindowEvent {
                event: WindowEvent::CursorMoved {
                    device_id: _,
                    position,
                    modifiers
                },
                window_id,
            } if window_id == window.id() => {
                pos = cgmath::vec2(position.x as f32, position.y as f32);
                //hovering = (cgmath::vec2(position.x, position.y) - cgmath::vec2(100.0, 100.0)).magnitude() < 90.0;
                window.request_redraw();
            },
            Event::WindowEvent {
                event: WindowEvent::MouseInput {
                    device_id: _,
                    button: button,
                    state: state,
                    modifiers: _,
                },
                window_id,
            } if window_id == window.id() => {
                if button == winit::event::MouseButton::Left{
                    hovering = state == winit::event::ElementState::Pressed;
                    window.request_redraw();
                }
            }
            _ => (),
        }
    });
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

    canvas.clear(if window.theme() == winit::window::Theme::Dark { skia_safe::Color::DARK_GRAY } else { skia_safe::Color::WHITE });

    let mut paint = skia_safe::Paint::default();
    paint.set_anti_alias(true);
    paint.set_color4f(if false {
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