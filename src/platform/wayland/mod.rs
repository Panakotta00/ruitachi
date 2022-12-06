use std::cmp::max;
use std::sync::{Arc, Mutex};
use raw_window_handle::{HasRawWindowHandle, RawWindowHandle, WaylandHandle};
use skia_safe::colors;
use wayland_client::protocol::wl_buffer::WlBuffer;
use wayland_client::protocol::wl_display::WlDisplay;
use winit::window::Window;
use wayland_client::protocol::wl_surface::WlSurface;
use wayland_client::{Attached, Display, EventQueue, GlobalManager, Main, Proxy};
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::platform::unix::{EventLoopWindowTargetExtUnix, WindowExtUnix};
use winit::platform::unix::x11::ffi::Connection;
use crate::paint;
use std::{fs::File, os::unix::prelude::AsRawFd};
use std::borrow::BorrowMut;
use std::io::{Read, Write};
use memmap2::Mmap;

use wayland_client::{
    protocol::{
        wl_buffer, wl_compositor, wl_keyboard, wl_registry, wl_seat, wl_shm, wl_shm_pool,
        wl_surface,
    },
};
use winit::platform::run_return::EventLoopExtRunReturn;

pub struct Context {
    wayland_event_queue: EventQueue,
    wayland_surface: Attached<WlSurface>,
    wayland_display: Attached<WlDisplay>,
    wayland_buffer: Option<Main<WlBuffer>>,
    wayland_globals: GlobalManager,
    temp_file: File,
}

impl<E> crate::platform::common::PlatformContext<E> for Context {
    fn new(window: &mut Window, event_loop: &mut EventLoop<E>) -> Context {
        let mut wayland_event_queue = event_loop.wayland_display().map(|display| {
            unsafe { Display::from_external_display(display as _) }.create_event_queue()
        }).unwrap();
        let surface = unsafe { Proxy::<WlSurface>::from_c_ptr(window.wayland_surface().unwrap() as _) };
        let display = unsafe { Proxy::<WlDisplay>::from_c_ptr(window.wayland_display().unwrap() as _) };
        let wayland_surface = surface.attach(wayland_event_queue.token());
        let wayland_display = display.attach(wayland_event_queue.token());

        let wayland_globals = GlobalManager::new(&wayland_display);

        wayland_event_queue.sync_roundtrip(&mut (), |_, _, _| unreachable!()).unwrap();

        let (buf_x, buf_y) = (320, 240);

        let mut temp = tempfile::tempfile().unwrap();
        let len = buf_x * buf_y * 4;
        let mut temp_arr = Vec::with_capacity(len as usize);
        temp_arr.resize(len as usize, 0);
        temp.write_all(temp_arr.borrow_mut()).unwrap();
        temp.flush();
        draw_wl(&mut temp, (buf_x, buf_y));

        let shm = wayland_globals.instantiate_exact::<wl_shm::WlShm>(1).unwrap();
        let pool = shm.create_pool(
            temp.as_raw_fd(),            // RawFd to the tempfile serving as shared memory
            (buf_x * buf_y * 4) as i32, // size in bytes of the shared memory (4 bytes per pixel)
        );
        let buffer = pool.create_buffer(
            0,                        // Start of the buffer in the pool
            buf_x as i32,             // width of the buffer in pixels
            buf_y as i32,             // height of the buffer in pixels
            (buf_x * 4) as i32,       // number of bytes between the beginning of two consecutive lines
            wl_shm::Format::Argb8888, // chosen encoding for the data
        );

        wayland_surface.commit();
        wayland_event_queue.sync_roundtrip(&mut (), |_, _, _| {}).expect("meep");
        println!("nice");
        wayland_surface.attach(Some(&buffer), 0, 0);
        println!("okay");
        wayland_surface.commit();
        println!("fuck");
        wayland_event_queue.sync_roundtrip(&mut (), |_, _, _| {}).expect("meep");
        println!("mhm");

        Self {
            wayland_event_queue,
            wayland_surface,
            wayland_display,
            wayland_buffer: Some(buffer),
            wayland_globals,
            temp_file: temp,
        }
    }

    fn run(&mut self, window: &mut Window, event_loop: &mut EventLoop<E>) {
        event_loop.run_return(move |event, _, control_flow| {
            *control_flow = ControlFlow::Wait;

            match event {
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    window_id,
                } if window_id == window.id() => *control_flow = ControlFlow::Exit,
                Event::RedrawRequested(_) => {

                    //draw_wl(&mut self.temp_file, (100, 100));



                    //self.wayland_surface.commit();
                    //self.DrawWindow(self, window, draw)
                },
                Event::WindowEvent {
                    event: WindowEvent::CursorMoved {
                        device_id: _,
                        position,
                        modifiers
                    },
                    window_id,
                } if window_id == window.id() => {
                    //pos = cgmath::vec2(position.x as f32, position.y as f32);
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
                    if button == winit::event::MouseButton::Left {
                        //hovering = state == winit::event::ElementState::Pressed;
                        window.request_redraw();
                    }
                },
                Event::RedrawEventsCleared => {
                    /*self.wayland_event_queue
                        .dispatch_pending(&mut (), || {

                        })
                        .expect("failed to dispatch wayland event queue");*/
                }
                _ => (),
            }
        });
    }

    fn draw_window(&mut self, window: &mut Window, func: fn(&mut Window, &mut crate::paint::Painter)) {
        /*let (bmpw, bmph) = (320, 240);
        let info = skia_safe::ImageInfo::new(
            (bmpw, bmph),
            skia_safe::ColorType::BGRA8888,
            skia_safe::AlphaType::Unpremul,
            None,
        );*/

        /*
        let mut info;
        let mut cs;
        unsafe {
            skia_bindings::C_SkImageInfo_Construct(&mut info);
            cs = skia_bindings::C_SkColorSpace_MakeSRGB();
            //skia_bindings::C_SkImageInfo_Make(&mut info, bmpw, bmph, kBGRA_8888_SkColorType, kPremul_SkAlphaType, &mut cs);
            info = skia_bindings::SkImageInfo::MakeS32(bmpw, bmph, skia_bindings::SkAlphaType::Premul);
        }*/

        /*let min_row_bytes = info.min_row_bytes();
        let mut surface = skia_safe::Surface::new_raster_direct(
            &info,
            pixels_p,
            Some(min_row_bytes),
            None,
        ).unwrap();

        func(window, &mut surface);

        unsafe {
            //windows::Win32::Graphics::Gdi::StretchDIBits(hdc, 0, 0, bmpw, bmph, 0, 0, bmpw, bmph, pixels.as_ptr() as *const c_void, bmpInfo, DIB_RGB_COLORS, SRCCOPY);

            //windows::Win32::Graphics::Gdi::EndPaint(hwnd, &paintStruct);
        }*/
    }
}

fn draw(window : &mut winit::window::Window, painter : &mut paint::Painter) {
    let canvas = painter.canvas();

    //canvas.clear(if window.theme() == winit::window::Theme::Dark { skia_safe::Color::DARK_GRAY } else { skia_safe::Color::WHITE });

    let mut paint = skia_safe::Paint::default();
    paint.set_anti_alias(true);
    /*paint.set_color4f(if unsafe { hovering } {
        colors::BLUE
    } else {
        colors::RED
    }, None);*/

    canvas.draw_circle((100, 100), 90.0, &paint);

    if let Some(text) = skia_safe::TextBlob::new("Hello World", &skia_safe::Font::default()) {
        canvas.draw_text_blob(text, (200, 100), &paint);
    }

    //canvas.draw_circle((pos.x, pos.y), 5.0, &paint);
}

fn draw_wl(tmp: &mut File, (buf_x, buf_y): (u32, u32)) {
    //use std::{cmp::min, io::Write};
    /*for y in 0..buf_y {
        for x in 0..buf_x {
            let a = 0xFF;
            let r = min(((buf_x - x) * 0xFF) / buf_x, ((buf_y - y) * 0xFF) / buf_y);
            let g = min((x * 0xFF) / buf_x, ((buf_y - y) * 0xFF) / buf_y);
            let b = min(((buf_x - x) * 0xFF) / buf_x, (y * 0xFF) / buf_y);

            let color = (a << 24) + (r << 16) + (g << 8) + b;
            buf.write_all(&color.to_ne_bytes()).unwrap();
        }
    }*/
    let (bmpw, bmph) = (320, 240);

    let info = skia_safe::ImageInfo::new(
        (bmpw, bmph),
        skia_safe::ColorType::BGRA8888,
        skia_safe::AlphaType::Unpremul,
        None,
    );

    let mut buf = unsafe { memmap2::MmapMut::map_mut(tmp.as_raw_fd()).unwrap() };

    let min_row_bytes = info.min_row_bytes();
    let mut surface = skia_safe::Surface::new_raster_direct(
        &info,
        buf.as_mut(),
        Some(min_row_bytes),
        None,
    ).unwrap();

    let canvas = surface.canvas();
    canvas.clear(skia_safe::Color::DARK_GRAY);
    let mut paint = skia_safe::Paint::default();
    paint.set_anti_alias(true);
    paint.set_color(skia_safe::Color::BLUE);
    canvas.draw_circle((100, 100), 90.0, &paint);
}