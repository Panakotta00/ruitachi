use crate::{
	events,
	events::{EventContext, WidgetEvent},
	platform::{common::PlatformContext, winit::WinitPlatformSpecifics},
	util::{Geometry, WidgetRef},
};
use cgmath::Vector2;
use raw_window_handle::{HasRawWindowHandle, RawWindowHandle, Win32Handle};
use skia_safe::scalar;
use std::{borrow::BorrowMut, cmp::max, ffi::c_void};
use windows::Win32::Graphics::Gdi::{
	BeginPaint, CreateCompatibleBitmap, EndPaint, SetDIBitsToDevice, BITMAPINFO, BITMAPINFOHEADER,
	BI_RGB, DIB_RGB_COLORS, PAINTSTRUCT, RGBQUAD, SRCCOPY,
};
use winit::{
	event::{ElementState, Event, MouseButton, WindowEvent},
	event_loop::{ControlFlow, EventLoop},
	platform::run_return::EventLoopExtRunReturn,
	window::Window,
};

pub use crate::platform::winit::*;

pub type Context = crate::platform::winit::Context<WindowsWinitSpecifics>;

pub struct WindowsWindowSpecificData {
	handle: Win32Handle,
	hwnd: windows::Win32::Foundation::HWND,
	bmp_info: Option<Vec<u8>>,
}

impl WindowsWindowSpecificData {
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

	pub fn resize_buffer(&mut self) {
		let (width, height) = self.get_size();

		let bmp_size = std::mem::size_of::<BITMAPINFOHEADER>()
			+ ((width * height) as usize) * std::mem::size_of::<RGBQUAD>();

		self.bmp_info = Some(vec![0 as u8; bmp_size]);

		let bmp_info =
			unsafe { &mut *(self.bmp_info.as_mut().unwrap().as_mut_ptr() as *mut BITMAPINFO) };

		unsafe {
			bmp_info.bmiHeader.biWidth = width;
			bmp_info.bmiHeader.biHeight = -height;
			bmp_info.bmiHeader.biSize = std::mem::size_of::<BITMAPINFOHEADER>() as u32;
			bmp_info.bmiHeader.biPlanes = 1;
			bmp_info.bmiHeader.biBitCount = 32;
			bmp_info.bmiHeader.biCompression = BI_RGB as u32;
		}
	}
}

pub struct WindowsWinitSpecifics;

impl WinitPlatformSpecifics for WindowsWinitSpecifics {
	type WindowSpecificData = WindowsWindowSpecificData;

	fn add_window(
		&mut self,
		window: &WidgetRef<dyn crate::widgets::Window>,
		winit_window: &mut Window,
		event_loop: &mut EventLoop<()>,
	) -> Self::WindowSpecificData {
		let handle = match winit_window.raw_window_handle() {
			RawWindowHandle::Win32(handle) => handle,
			_ => panic!("Windows Platform Specific function called!"),
		};

		let hwnd = windows::Win32::Foundation::HWND(handle.hwnd as isize);

		let mut specific_data = WindowsWindowSpecificData {
			hwnd,
			handle,
			bmp_info: None,
		};

		specific_data.resize_buffer();

		specific_data
	}

	fn remove_window(
		&mut self,
		window: WidgetRef<crate::platform::winit::Window<Self::WindowSpecificData>>,
	) {
	}

	fn resize_window(
		&mut self,
		window: WidgetRef<crate::platform::winit::Window<Self::WindowSpecificData>>,
	) {
		window.get().platform_specific_data.resize_buffer();
	}

	fn flush_window_buffer(
		&mut self,
		window: WidgetRef<crate::platform::winit::Window<Self::WindowSpecificData>>,
	) {
		let mut window = window.get();
		let hwnd = window.platform_specific_data.hwnd;
		let bmp_info = unsafe {
			&mut *(window
				.platform_specific_data
				.bmp_info
				.as_mut()
				.unwrap()
				.as_mut_ptr() as *mut BITMAPINFO)
		};
		unsafe {
			windows::Win32::Graphics::Gdi::InvalidateRect(hwnd, std::ptr::null(), true);
			let mut paint_struct = PAINTSTRUCT::default();
			let hdc = BeginPaint(hwnd, &mut paint_struct);
			let bitmap = &mut window.skia_data.as_mut().unwrap().1;
			SetDIBitsToDevice(
				hdc,
				0,
				0,
				bitmap.width() as u32,
				bitmap.height() as u32,
				0,
				0,
				0,
				bitmap.height() as u32,
				bitmap.pixels(),
				bmp_info,
				DIB_RGB_COLORS,
			);
			EndPaint(hwnd, &paint_struct);
		}
	}
}

pub fn create_platform() -> Context {
	Context::new(WindowsWinitSpecifics {})
}
