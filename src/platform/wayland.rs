use crate::util::WidgetRef;

use std::{cell::RefMut, fs::File, os::unix::prelude::AsRawFd};
use wayland_client::{
	protocol::{
		wl_buffer::WlBuffer, wl_display::WlDisplay, wl_shm, wl_shm_pool::WlShmPool,
		wl_surface::WlSurface,
	},
	Attached, Display, EventQueue, GlobalManager, Main, Proxy,
};
use winit::{
	dpi::PhysicalSize,
	event_loop::EventLoop,
	platform::unix::{EventLoopWindowTargetExtUnix, WindowExtUnix},
};

pub use crate::platform::winit::*;

pub type Context = crate::platform::winit::Context<WaylandWinitSpecifics>;

pub struct WaylandWindowSpecificData {
	temp_file: File,
	wayland_event_queue: EventQueue,
	wayland_surface: Attached<WlSurface>,
	wayland_display: Attached<WlDisplay>,
	wayland_pool: Option<Main<WlShmPool>>,
	wayland_globals: GlobalManager,
	wayland_buffer: Option<Main<WlBuffer>>,
	buffer_map: Option<memmap2::MmapMut>,
}

impl WaylandWindowSpecificData {
	pub fn resize_buffer(&mut self, window: &mut winit::window::Window) {
		let size = window.inner_size();

		let buffer_size = size.width * size.height * 4;

		// resize buffer
		self.temp_file.set_len(buffer_size as u64).unwrap();

		self.buffer_map = Some(unsafe {
			memmap2::MmapMut::map_mut(self.temp_file.as_raw_fd())
				.expect("Unable to map draw-buffer to memory")
		});

		// Create Wayland Draw Buffer
		let shm = self
			.wayland_globals
			.instantiate_exact::<wl_shm::WlShm>(1)
			.unwrap();
		self.wayland_pool = Some(shm.create_pool(self.temp_file.as_raw_fd(), buffer_size as i32));

		self.wayland_buffer = Some(self.wayland_pool.as_mut().unwrap().create_buffer(
			0,
			size.width as i32,
			size.height as i32,
			size.width as i32 * 4,
			wl_shm::Format::Argb8888,
		));

		// Flush to wayland
		self.wayland_surface
			.attach(Some(self.wayland_buffer.as_ref().unwrap()), 0, 0);
		self.wayland_event_queue
			.sync_roundtrip(&mut (), |_, _, _| {})
			.expect("meep");
		/* else {
			self.wayland_pool
				.as_mut()
				.unwrap()
				.resize(buffer_size as i32);
		}*/
	}
}

pub struct WaylandWinitSpecifics;

impl WinitPlatformSpecifics for WaylandWinitSpecifics {
	type WindowSpecificData = WaylandWindowSpecificData;

	fn add_window(
		&mut self,
		_window: &WidgetRef<dyn crate::widgets::Window>,
		winit_window: &mut winit::window::Window,
		event_loop: &mut EventLoop<()>,
	) -> Self::WindowSpecificData {
		// Create Wayland connection and get necessary globals
		let mut wayland_event_queue = event_loop
			.wayland_display()
			.map(|display| {
				unsafe { Display::from_external_display(display as _) }.create_event_queue()
			})
			.unwrap();
		let surface =
			unsafe { Proxy::<WlSurface>::from_c_ptr(winit_window.wayland_surface().unwrap() as _) };
		let display =
			unsafe { Proxy::<WlDisplay>::from_c_ptr(winit_window.wayland_display().unwrap() as _) };
		let wayland_surface = surface.attach(wayland_event_queue.token());
		let wayland_display = display.attach(wayland_event_queue.token());

		let wayland_globals = GlobalManager::new(&wayland_display);
		wayland_event_queue
			.sync_roundtrip(&mut (), |_, _, _| unreachable!())
			.unwrap();

		// Create Draw buffer
		let temp_file = tempfile::tempfile().unwrap();

		let mut specific_data = WaylandWindowSpecificData {
			temp_file,
			wayland_event_queue,
			wayland_surface,
			wayland_display,
			wayland_globals,
			wayland_pool: None,
			wayland_buffer: None,
			buffer_map: None,
		};
		specific_data
	}

	fn remove_window(&mut self, window: WidgetRef<Window<Self::WindowSpecificData>>) {
		let data = &mut window.get().platform_specific_data;
		data.buffer_map = None;
		if let Some(buf) = &data.wayland_buffer {
			buf.destroy();
			data.wayland_buffer = None;
		}
		if let Some(pool) = &data.wayland_pool {
			pool.destroy();
			data.wayland_pool = None;
		}
		data.wayland_event_queue
			.sync_roundtrip(&mut (), |_, _, _| unreachable!())
			.unwrap();
	}

	fn resize_buffer(&mut self, window: WidgetRef<Window<Self::WindowSpecificData>>) {
		let window = window.get();
		let (mut data, mut window) = RefMut::map_split(window, |w| {
			(&mut w.platform_specific_data, &mut w.winit_window)
		});

		data.resize_buffer(&mut window);
	}

	fn flush_window_buffer(&mut self, window: WidgetRef<Window<Self::WindowSpecificData>>) {
		let (mut specific_data, mut winit_window) = RefMut::map_split(window.get(), |w| {
			(&mut w.platform_specific_data, &mut w.winit_window)
		});
		specific_data.resize_buffer(&mut winit_window);
		drop((specific_data, winit_window));
		let window = window.get();
		let size = window.size;
		let (skia_data, mut specific_data) = RefMut::map_split(window, |w| {
			(&mut w.skia_data, &mut w.platform_specific_data)
		});

		let data = skia_data.as_ref().unwrap().1.pixmap().bytes();

		specific_data
			.buffer_map
			.as_mut()
			.unwrap()
			.copy_from_slice(data.unwrap());

		specific_data.wayland_surface.commit();
		specific_data.wayland_surface.attach(
			Some(specific_data.wayland_buffer.as_ref().unwrap()),
			0,
			0,
		);
		specific_data
			.wayland_surface
			.damage(0, 0, size.width as i32, size.height as i32);
		specific_data
			.wayland_event_queue
			.sync_roundtrip(&mut (), |_, _, _| {})
			.unwrap();
	}
}

pub fn create_platform() -> Context {
	Context::new(WaylandWinitSpecifics {})
}
