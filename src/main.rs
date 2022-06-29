extern crate core;

use std::os::raw::c_void;
use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};
use skia_bindings::SkColorType::kBGRA_8888_SkColorType;
use windows::Win32::Graphics::Gdi::{BeginPaint, BI_RGB, BITMAPINFO, BITMAPINFOHEADER, DIB_RGB_COLORS, PAINTSTRUCT, SRCCOPY};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use winit::platform::windows::WindowBuilderExtWindows;

fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Hello World")
        .with_decorations(true)
        .with_transparent(true)
        .with_theme(Some(winit::window::Theme::Dark))
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
                draw(&window)
            },
            _ => (),
        }
    });
}

fn draw(window : &winit::window::Window) {
    unsafe {
        match window.raw_window_handle() {
            RawWindowHandle::Win32(handle) => {
                let hwnd = windows::Win32::Foundation::HWND(handle.hwnd as isize);
                let mut rect = windows::Win32::Foundation::RECT {
                    bottom: 0,
                    left: 0,
                    top: 0,
                    right: 0
                };
                if unsafe { windows::Win32::UI::WindowsAndMessaging::GetClientRect(hwnd, &mut rect) } == windows::Win32::Foundation::BOOL(0) {
                    return
                }
                let bmpw = rect.right - rect.left;
                let bmph = rect.bottom - rect.top;

                let mut paintStruct = PAINTSTRUCT::default();
                let hdc = unsafe { BeginPaint(hwnd, &mut paintStruct) };
                let bmpSize = std::mem::size_of::<BITMAPINFOHEADER>() + ((bmpw * bmph) as usize) * std::mem::size_of::<u32>();
                let bmpInfo = vec![0 as u8; bmpSize].as_mut_ptr() as *mut BITMAPINFO;
                unsafe {
                    (*bmpInfo).bmiHeader.biSize = std::mem::size_of::<BITMAPINFOHEADER>() as u32;
                    (*bmpInfo).bmiHeader.biWidth = bmpw;
                    (*bmpInfo).bmiHeader.biHeight = -bmph;
                    (*bmpInfo).bmiHeader.biPlanes = 1;
                    (*bmpInfo).bmiHeader.biBitCount = 32;
                    (*bmpInfo).bmiHeader.biCompression = BI_RGB as u32;
                }

                let mut pixels = unsafe { (*bmpInfo).bmiColors };
                let pixels_p = &mut *(pixels.as_mut_ptr() as *mut [u8; 1]);

                let info = skia_safe::ImageInfo::new(
                    (bmpw, bmph),
                    skia_safe::ColorType::RGBA8888,
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

                windows::Win32::Graphics::Gdi::StretchDIBits(hdc, 0, 0, bmpw, bmph, 0, 0, bmpw, bmph, pixels.as_ptr() as *const c_void, bmpInfo, DIB_RGB_COLORS, SRCCOPY);

                windows::Win32::Graphics::Gdi::EndPaint(hwnd, &paintStruct);
            }
            _ => {}
        }
    }
}