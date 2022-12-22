use crate::util::{Geometry, WidgetRef};
use raw_window_handle::{HasRawWindowHandle, RawWindowHandle, Win32Handle};
use std::borrow::BorrowMut;
use std::cmp::max;
use std::ffi::c_void;
use cgmath::Vector2;
use skia_safe::scalar;
use windows::Win32::Graphics::Gdi::{
	BeginPaint, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS, PAINTSTRUCT, RGBQUAD, SRCCOPY,
};
use winit::event::{ElementState, Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::platform::run_return::EventLoopExtRunReturn;
use winit::window::Window;
use crate::events;
use crate::events::{EventContext, WidgetEvent};

pub struct Context {
	window_widget: WidgetRef<dyn crate::widgets::Window>,
	handle: Win32Handle,
	hwnd: windows::Win32::Foundation::HWND,
	size: (i32, i32),
	bmp_info: Option<Vec<u8>>,
	last_cursor_pos: Vector2<scalar>,
	event_context: EventContext,
}

impl Context {
	pub fn resize(&mut self, (width, height): (i32, i32)) {
		let bmpSize = std::mem::size_of::<BITMAPINFOHEADER>()
			+ ((width * height) as usize) * std::mem::size_of::<RGBQUAD>();
		self.bmp_info = Some(vec![0 as u8; bmpSize]);
		let bmpInfo =
			unsafe { &mut *(self.bmp_info.as_mut().unwrap().as_mut_ptr() as *mut BITMAPINFO) };
		unsafe {
			bmpInfo.bmiHeader.biWidth = width;
			bmpInfo.bmiHeader.biHeight = -height;
			bmpInfo.bmiHeader.biSize = std::mem::size_of::<BITMAPINFOHEADER>() as u32;
			bmpInfo.bmiHeader.biPlanes = 1;
			bmpInfo.bmiHeader.biBitCount = 32;
			bmpInfo.bmiHeader.biCompression = BI_RGB as u32;
		}
		//let mut pixels = unsafe {&(*bmpInfo).bmiColors };

		self.size = (width, height);
	}

	pub fn get_size(&self) -> (i32, i32) {
		let mut rect = windows::Win32::Foundation::RECT {
			bottom: 0,
			left: 0,
			top: 0,
			right: 0,
		};
		unsafe {
			windows::Win32::UI::WindowsAndMessaging::GetClientRect(self.hwnd, &mut rect);
		}
		let width = max(rect.right - rect.left, 1);
		let height = max(rect.bottom - rect.top, 1);
		(width, height)
	}
}

impl<E> crate::platform::common::PlatformContext<E> for Context {
	fn new(
		window: &mut Window,
		event_loop: &mut EventLoop<E>,
		window_widget: WidgetRef<dyn crate::widgets::Window>,
	) -> Self {
		let handle = match window.raw_window_handle() {
			RawWindowHandle::Win32(handle) => handle,
			_ => panic!("Windows Platform Specific function called!"),
		};

		let hwnd = windows::Win32::Foundation::HWND(handle.hwnd as isize);

		let size = window.outer_size();

		let mut context = Self {
			window_widget,
			handle,
			hwnd,
			bmp_info: None,
			size: (0, 0),
			last_cursor_pos: Vector2::new(0.0, 0.0),
			event_context: EventContext::new(),
		};

		context.resize((size.width as i32, size.height as i32));

		context
	}

	fn run(&mut self, window: &mut Window, event_loop: &mut EventLoop<E>) {
		event_loop.run_return(|event, _, control_flow| {
			*control_flow = ControlFlow::Wait;

			match event {
				Event::WindowEvent {
					event: WindowEvent::CloseRequested,
					window_id,
				} if window_id == window.id() => *control_flow = ControlFlow::Exit,
				Event::WindowEvent {
					event: WindowEvent::Resized(size),
					window_id,
				} if window_id == window.id() => {
					self.resize((size.width as i32, size.height as i32));
				}
				Event::RedrawRequested(_) => {
					let bmp_info = unsafe {
						&mut *(self.bmp_info.as_mut().unwrap().as_mut_ptr() as *mut BITMAPINFO)
					};

					let pixels_ptr = bmp_info.bmiColors.as_mut_ptr() as *const c_void;
					let pixels = unsafe {
						let mut ptr = pixels_ptr as *mut u8;
						std::slice::from_raw_parts_mut(
							ptr,
							(self.size.0 * self.size.1) as usize * std::mem::size_of::<RGBQUAD>(),
						)
					};

					// Create Skia Context/Image/Buffer/Surface
					let skia_info = skia_safe::ImageInfo::new(
						self.size,
						skia_safe::ColorType::BGRA8888,
						skia_safe::AlphaType::Unpremul,
						None,
					);
					let min_row_bytes = skia_info.min_row_bytes();
					let mut skia_surface = skia_safe::Surface::new_raster_direct(
						&skia_info,
						pixels,
						Some(min_row_bytes),
						None,
					)
					.expect("Unable to create Skia drawing surface");

					let mut window = self.window_widget.get();
					window.draw(&mut skia_surface);

					unsafe {
						windows::Win32::Graphics::Gdi::InvalidateRect(
							self.hwnd,
							std::ptr::null(),
							true,
						);
						let mut paint_struct = PAINTSTRUCT::default();
						let hdc = BeginPaint(self.hwnd, &mut paint_struct);
						windows::Win32::Graphics::Gdi::StretchDIBits(
							hdc,
							0,
							0,
							self.size.0,
							self.size.1,
							0,
							0,
							self.size.0,
							self.size.1,
							pixels_ptr,
							bmp_info,
							DIB_RGB_COLORS,
							SRCCOPY,
						);
						windows::Win32::Graphics::Gdi::EndPaint(self.hwnd, &paint_struct);
					}
				}
				Event::WindowEvent {
					event:
						WindowEvent::CursorMoved {
							device_id: _,
							position,
							modifiers: _,
						},
					window_id,
				} if window_id == window.id() => {
					let pos = Vector2::new(position.x as f32, position.y as f32);
					self.last_cursor_pos = pos;
					let size = window.inner_size();
					let geometry = Geometry::new(
						Vector2::new(0.0, 0.0),
						Vector2::new(size.width as scalar, size.height as scalar),
						Vector2::new(0.0, 0.0),
						Vector2::new(1.0, 1.0),
					);

					// TODO: Add multi device support
					let path = events::get_widget_path_under_position(geometry, self.window_widget.clone(), &pos);
					self.event_context.handle_mouse_move(&path, 0, &self.last_cursor_pos);

					window.request_redraw();
				}
				Event::WindowEvent {
					event:
						WindowEvent::MouseInput {
							device_id: _,
							button,
							state,
							modifiers: _,
						},
					window_id,
				} if window_id == window.id() => {
					if button == winit::event::MouseButton::Left {
						let size = window.inner_size();
						let geometry = Geometry::new(
							Vector2::new(0.0, 0.0),
							Vector2::new(size.width as scalar, size.height as scalar),
							Vector2::new(0.0, 0.0),
							Vector2::new(1.0, 1.0),
						);

						// TODO: Add multi device support
						let path = events::get_widget_path_under_position(geometry, self.window_widget.clone(), &self.last_cursor_pos);
						match state {
							ElementState::Pressed => self.event_context.handle_mouse_button_down(&path, 0, &self.last_cursor_pos),
							ElementState::Released => self.event_context.handle_mouse_button_up(&path, 0, &self.last_cursor_pos),
						}


						window.request_redraw();
					}
				},
				Event::WindowEvent {
					window_id,
					event: WindowEvent::KeyboardInput {
						device_id,
						input,
						is_synthetic
					}
				} if window_id == window.id() => {
					match input.state {
						// TODO: Add multi device support
						ElementState::Pressed => self.event_context.handle_key_down(0),
						ElementState::Released => self.event_context.handle_key_up(0),
					}
				}
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
}
