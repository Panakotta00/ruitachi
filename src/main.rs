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

fn main() {
    let mut hovering = false;
    let mut pos = cgmath::Vector2::<f32>::new(0.0, 0.0);
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Hello World")
        .with_decorations(true)
        .with_transparent(true)
        .build(&event_loop)
        .unwrap();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id,
            } if window_id == window.id() => *control_flow = ControlFlow::Exit,
            Event::RedrawRequested(_) => {
                draw(&window, hovering, pos);
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

fn draw(window : &winit::window::Window, hovering : bool, pos : cgmath::Vector2<f32>) {
    unsafe {
        match window.raw_window_handle() {
            RawWindowHandle::Win32(handle) => {
                let hwnd = windows::Win32::Foundation::HWND(handle.hwnd as isize);
                windows::Win32::Graphics::Gdi::InvalidateRect(hwnd, std::ptr::null(), false);
                let mut rect = windows::Win32::Foundation::RECT {
                    bottom: 0,
                    left: 0,
                    top: 0,
                    right: 0
                };
                if unsafe { windows::Win32::UI::WindowsAndMessaging::GetClientRect(hwnd, &mut rect) } == windows::Win32::Foundation::BOOL(0) {
                    return
                }
                let bmpw = max(rect.right - rect.left, 1);
                let bmph = max(rect.bottom - rect.top, 1);

                let mut paintStruct = PAINTSTRUCT::default();
                let hdc = unsafe { BeginPaint(hwnd, &mut paintStruct) };
                let bmpSize = std::mem::size_of::<BITMAPINFOHEADER>() + ((bmpw * bmph) as usize) * std::mem::size_of::<RGBQUAD>();
                let mut bmpInfoVec = vec![0 as u8; bmpSize];
                let bmpInfo = bmpInfoVec.as_mut_ptr() as *mut BITMAPINFO;
                unsafe {
                    (*bmpInfo).bmiHeader.biWidth = bmpw;
                    (*bmpInfo).bmiHeader.biHeight = -bmph;
                    (*bmpInfo).bmiHeader.biSize = std::mem::size_of::<BITMAPINFOHEADER>() as u32;
                    (*bmpInfo).bmiHeader.biPlanes = 1;
                    (*bmpInfo).bmiHeader.biBitCount = 32;
                    (*bmpInfo).bmiHeader.biCompression = BI_RGB as u32;
                }

                let mut pixels = unsafe {&(*bmpInfo).bmiColors };
                let mut pixels2 = (*bmpInfo).bmiColors.as_mut_ptr() as *mut u8;
                let pixels_p = std::slice::from_raw_parts_mut(pixels2, (bmpw * bmph) as usize * std::mem::size_of::<RGBQUAD>());

                let info = skia_safe::ImageInfo::new(
                    (bmpw, bmph),
                    skia_safe::ColorType::BGRA8888,
                    skia_safe::AlphaType::Unpremul,
                    None,
                );

                /*
                let mut info;
                let mut cs;
                unsafe {
                    skia_bindings::C_SkImageInfo_Construct(&mut info);
                    cs = skia_bindings::C_SkColorSpace_MakeSRGB();
                    //skia_bindings::C_SkImageInfo_Make(&mut info, bmpw, bmph, kBGRA_8888_SkColorType, kPremul_SkAlphaType, &mut cs);
                    info = skia_bindings::SkImageInfo::MakeS32(bmpw, bmph, skia_bindings::SkAlphaType::Premul);
                }*/

                let min_row_bytes = info.min_row_bytes();
                let mut surface = skia_safe::Surface::new_raster_direct(
                    &info,
                    pixels_p,
                    Some(min_row_bytes),
                    None,
                ).unwrap();

                let canvas = surface.canvas();

                canvas.clear(if window.theme() == winit::window::Theme::Dark { skia_safe::Color::DARK_GRAY } else { skia_safe::Color::WHITE });
                let mut paint = skia_safe::Paint::default();
                paint.set_color4f(if hovering {
                    colors::BLUE
                } else {
                    colors::RED
                }, None);
                paint.set_anti_alias(true);
                canvas.draw_circle((100, 100), 90.0, &paint);
                if let Some(text) = skia_safe::TextBlob::new("Hello World", &skia_safe::Font::default()) {
                    canvas.draw_text_blob(text, (200, 100), &paint);
                }
                canvas.draw_circle((pos.x, pos.y), 5.0, &paint);

                windows::Win32::Graphics::Gdi::StretchDIBits(hdc, 0, 0, bmpw, bmph, 0, 0, bmpw, bmph, pixels.as_ptr() as *const c_void, bmpInfo, DIB_RGB_COLORS, SRCCOPY);

                windows::Win32::Graphics::Gdi::EndPaint(hwnd, &paintStruct);
            }
            _ => {
                println!("Lul");
            }
        }
    }
}