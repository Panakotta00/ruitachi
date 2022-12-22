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
use std::borrow::BorrowMut;
use std::io::{Seek, SeekFrom, Write};
use std::{fs::File, os::unix::prelude::AsRawFd};
use cgmath::Vector2;
use skia_safe::scalar;

use wayland_client::protocol::wl_shm;
use wayland_client::protocol::wl_shm_pool::WlShmPool;
use winit::platform::run_return::EventLoopExtRunReturn;
use crate::events::WidgetEvent;
use crate::util::Geometry;

pub struct Context {
	window_widget: crate::util::WidgetRef<dyn crate::widgets::Window>,
	temp_file: File,
	wayland_event_queue: EventQueue,
	wayland_surface: Attached<WlSurface>,
	wayland_display: Attached<WlDisplay>,
	wayland_pool: Option<Main<WlShmPool>>,
	wayland_globals: GlobalManager,
	wayland_buffer: Option<Main<WlBuffer>>,
	buffer_map: Option<memmap2::MmapMut>,
	size: (i32, i32),
	last_cursor_pos: Vector2<scalar>,
}

impl Context {
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

impl<E> crate::platform::common::PlatformContext<E> for Context {
	fn new(
		window: &mut Window,
		event_loop: &mut EventLoop<E>,
		window_widget: crate::util::WidgetRef<dyn crate::widgets::Window>,
	) -> Self {
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

		let mut context = Self {
			window_widget,
			temp_file: temp,
			wayland_event_queue,
			wayland_surface,
			wayland_display,
			wayland_globals,
			wayland_pool: None,
			wayland_buffer: None,
			buffer_map: None,
			size: (size.width as i32, size.height as i32),
			last_cursor_pos: Vector2::new(0.0, 0.0),
		};

		context.resize((size.width as i32, size.height as i32));

		context
	}

	fn run(&mut self, window: &mut Window, event_loop: &mut EventLoop<E>) {
		event_loop.run_return(move |event, _, control_flow| {
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

					let mut window = self.window_widget.get();
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

					let path = events::get_widget_path_under_position(geometry, self.window_widget.clone(), &pos);
					let event = WidgetEvent::OnMouseMove;
					let reply = events::bubble_event(path, &event);

					window.request_redraw();
				}
				Event::WindowEvent {
					event:
						WindowEvent::MouseInput {
							device_id: _,
							button,
							state: _,
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

						let path = events::get_widget_path_under_position(geometry, self.window_widget.clone(), &self.last_cursor_pos);
						let event = WidgetEvent::OnMouseInput;
						let reply = events::bubble_event(path, &event);

						window.request_redraw();
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
