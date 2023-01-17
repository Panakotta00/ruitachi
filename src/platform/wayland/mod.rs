use wayland_client::{
	protocol::{wl_buffer::WlBuffer, wl_display::WlDisplay, wl_surface::WlSurface},
	Attached, Display, EventQueue, GlobalManager, Main, Proxy,
};
use winit::{
	event::{Event, WindowEvent},
	event_loop::{ControlFlow, EventLoop},
	platform::unix::{EventLoopWindowTargetExtUnix, WindowExtUnix},
	window::Window,
};

use crate::{events, paint};
use cgmath::Vector2;
use skia_safe::scalar;
use std::borrow::BorrowMut;
use std::io::{Seek, SeekFrom, Write};
use std::{fs::File, os::unix::prelude::AsRawFd};

use crate::events::{EventContext, WidgetEvent};
use crate::platform::common::PlatformContext;
use crate::util::{Geometry, WidgetRef};
use wayland_client::protocol::wl_shm;
use wayland_client::protocol::wl_shm_pool::WlShmPool;
use winit::event::{ElementState, MouseButton};
use winit::platform::run_return::EventLoopExtRunReturn;
use winit::window::CursorGrabMode;

pub struct Context<'a, E: 'static> {
	pub window_widget: Option<WidgetRef<dyn crate::widgets::Window>>,
	temp_file: File,
	wayland_event_queue: EventQueue,
	event_loop: Option<&'a mut EventLoop<E>>,
	wayland_surface: Attached<WlSurface>,
	wayland_display: Attached<WlDisplay>,
	wayland_pool: Option<Main<WlShmPool>>,
	wayland_globals: GlobalManager,
	wayland_buffer: Option<Main<WlBuffer>>,
	buffer_map: Option<memmap2::MmapMut>,
	size: (i32, i32),
	last_cursor_pos: Vector2<scalar>,
	window: &'a mut Window,
}

impl<'a, E> Context<'a, E> {
	fn resize(&mut self, (width, height): (i32, i32)) {
		let buffer_size = width * height * 4;
		let old_buffer_size = self.size.0 * self.size.1 * 4;

		// resize buffer
		self.temp_file.set_len(buffer_size as u64).unwrap();

		self.buffer_map = Some(unsafe {
			memmap2::MmapMut::map_mut(self.temp_file.as_raw_fd())
				.expect("Unable to map draw-buffer to memory")
		});

		// Resize wayland pool & buffer
		if old_buffer_size > buffer_size || self.wayland_pool.is_none() {
			// Create Wayland Draw Buffer
			let shm = self
				.wayland_globals
				.instantiate_exact::<wl_shm::WlShm>(1)
				.unwrap();
			self.wayland_pool = Some(shm.create_pool(self.temp_file.as_raw_fd(), buffer_size));
		} else {
			self.wayland_pool.as_mut().unwrap().resize(buffer_size);
		}
		self.wayland_buffer = Some(self.wayland_pool.as_mut().unwrap().create_buffer(
			0,
			width,
			height,
			width * 4,
			wl_shm::Format::Argb8888,
		));

		// Flush to wayland
		self.wayland_surface
			.attach(Some(self.wayland_buffer.as_ref().unwrap()), 0, 0);
		self.wayland_event_queue
			.sync_roundtrip(&mut (), |_, _, _| {})
			.expect("meep");

		self.size = (width, height);
	}
}

pub fn conv_mouse_button(btn: winit::event::MouseButton) -> events::input::MouseButton {
	match btn {
		MouseButton::Left => events::input::MouseButton::Left,
		MouseButton::Right => events::input::MouseButton::Right,
		MouseButton::Middle => events::input::MouseButton::Middle,
		MouseButton::Other(c) => events::input::MouseButton::Other(c),
	}
}

impl<'a, E> PlatformContext for Context<'a, E> {
	fn add_window(&mut self, window: &WidgetRef<dyn crate::widgets::Window>) {
		self.window_widget = Some(window.clone());
		self.resize(self.size);
	}

	fn run(&mut self, event_context: &mut EventContext) {
		let mut event = None;
		std::mem::swap(&mut event, &mut self.event_loop);
		event.unwrap().run_return(|event, _, control_flow| {
			*control_flow = ControlFlow::Wait;

			match event {
				Event::WindowEvent {
					event: WindowEvent::CloseRequested,
					window_id,
				} if window_id == self.window.id() => *control_flow = ControlFlow::Exit,
				Event::WindowEvent {
					event: WindowEvent::Resized(size),
					window_id,
				} if window_id == self.window.id() => {
					self.resize((size.width as i32, size.height as i32));
				}
				Event::RedrawRequested(_) => {
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
						self.buffer_map.as_mut().unwrap(),
						Some(min_row_bytes),
						None,
					)
					.expect("Unable to create Skia drawing surface");

					let mut window = self.window_widget.as_ref().unwrap().get();
					window.draw(&mut skia_surface);

					self.wayland_surface.commit();
					self.wayland_surface
						.attach(Some(self.wayland_buffer.as_ref().unwrap()), 0, 0);
					self.wayland_surface.damage(0, 0, self.size.0, self.size.1);
					self.wayland_event_queue
						.sync_roundtrip(&mut (), |_, _, _| {})
						.unwrap();
				}
				Event::WindowEvent {
					event:
						WindowEvent::CursorMoved {
							device_id: device_id,
							position,
							modifiers: _,
						},
					window_id,
				} if window_id == self.window.id() => {
					let pos = Vector2::new(position.x as f32, position.y as f32);
					self.last_cursor_pos = pos;
					let size = self.window.inner_size();
					let geometry = Geometry::new(
						Vector2::new(0.0, 0.0),
						Vector2::new(size.width as scalar, size.height as scalar),
						Vector2::new(0.0, 0.0),
						Vector2::new(1.0, 1.0),
					);

					// TODO: Add multi device support
					let path = events::get_widget_path_under_position(
						geometry,
						self.window_widget.as_ref().unwrap().clone(),
						&pos,
					);
					event_context.handle_mouse_move(&path, 0, &self.last_cursor_pos);

					self.window.request_redraw();
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
				} if window_id == self.window.id() => {
					if button == winit::event::MouseButton::Left {
						let size = self.window.inner_size();
						let geometry = Geometry::new(
							Vector2::new(0.0, 0.0),
							Vector2::new(size.width as scalar, size.height as scalar),
							Vector2::new(0.0, 0.0),
							Vector2::new(1.0, 1.0),
						);

						let path = events::get_widget_path_under_position(
							geometry,
							self.window_widget.as_ref().unwrap().clone(),
							&self.last_cursor_pos,
						);
						let pos = self.last_cursor_pos;
						match state {
							ElementState::Pressed => event_context.handle_mouse_button_down(
								self,
								&path,
								0,
								conv_mouse_button(button),
								&pos,
							),
							ElementState::Released => event_context.handle_mouse_button_up(
								self,
								&path,
								0,
								conv_mouse_button(button),
								&pos,
							),
						}

						self.window.request_redraw();
					}
				}
				Event::WindowEvent {
					window_id,
					event:
						WindowEvent::KeyboardInput {
							device_id,
							input,
							is_synthetic,
						},
				} if window_id == self.window.id() => {
					match input.state {
						// TODO: Add multi device support
						ElementState::Pressed => {
							event_context.handle_key_down(
								0,
								input.scancode as usize,
								input.virtual_keycode,
							);
						}
						ElementState::Released => {
							event_context.handle_key_up(
								0,
								input.scancode as usize,
								input.virtual_keycode,
							);
						}
					}
					self.window.request_redraw();
				}
				Event::WindowEvent {
					window_id,
					event: WindowEvent::ReceivedCharacter(char),
				} if window_id == self.window.id() => {
					// TODO: Add multi device support
					event_context.handle_text(0, char);
					self.window.request_redraw();
				}
				Event::WindowEvent {
					window_id,
					event: WindowEvent::CursorLeft { device_id },
				} if window_id == self.window.id() => {
					// TODO: Add multi device support
					event_context.handle_cursor_leave(0);
					self.window.request_redraw();
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

	fn set_capture_cursor(&mut self, cursor: usize, should_capture: bool) {}
}

impl<'a, E> Context<'a, E> {
	pub fn new(window: &'a mut Window, event_loop: &'a mut EventLoop<E>) -> Self {
		// Create Wayland connection and get necessary globals
		let mut wayland_event_queue = event_loop
			.wayland_display()
			.map(|display| {
				unsafe { Display::from_external_display(display as _) }.create_event_queue()
			})
			.unwrap();
		let surface =
			unsafe { Proxy::<WlSurface>::from_c_ptr(window.wayland_surface().unwrap() as _) };
		let display =
			unsafe { Proxy::<WlDisplay>::from_c_ptr(window.wayland_display().unwrap() as _) };
		let wayland_surface = surface.attach(wayland_event_queue.token());
		let wayland_display = display.attach(wayland_event_queue.token());

		let wayland_globals = GlobalManager::new(&wayland_display);
		wayland_event_queue
			.sync_roundtrip(&mut (), |_, _, _| unreachable!())
			.unwrap();

		// Create Draw buffer
		let mut temp = tempfile::tempfile().unwrap();

		let size = window.outer_size();

		Self {
			window_widget: None,
			temp_file: temp,
			wayland_event_queue,
			event_loop: Some(event_loop),
			wayland_surface,
			wayland_display,
			wayland_globals,
			wayland_pool: None,
			wayland_buffer: None,
			buffer_map: None,
			size: (size.width as i32, size.height as i32),
			last_cursor_pos: Vector2::new(0.0, 0.0),
			window,
		}
	}
}
