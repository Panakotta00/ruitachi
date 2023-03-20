pub use crate::platform::winit::*;
use crate::{platform::winit::WinitPlatformSpecifics, util::WidgetRef};
use raw_window_handle::{HasRawWindowHandle, RawWindowHandle, Win32Handle};
use std::cmp::max;
use windows::Win32::Graphics::Gdi::{
	BeginPaint, EndPaint, SetDIBitsToDevice, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS,
	PAINTSTRUCT,
};
use winit::event_loop::EventLoop;

pub type Context = crate::platform::winit::Context<WindowsWinitSpecifics>;

pub struct WindowsWindowSpecificData {
	handle: Win32Handle,
	hwnd: windows::Win32::Foundation::HWND,
	bmp_info_o: Option<Vec<u8>>,
	bmp_info: Option<BITMAPINFO>,
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

		let mut bmp_info = BITMAPINFO::default();
		bmp_info.bmiHeader.biSize = std::mem::size_of::<BITMAPINFOHEADER>() as u32;
		bmp_info.bmiHeader.biWidth = width;
		bmp_info.bmiHeader.biHeight = -height;
		bmp_info.bmiHeader.biPlanes = 1;
		bmp_info.bmiHeader.biBitCount = 32;
		bmp_info.bmiHeader.biCompression = BI_RGB as u32;
		bmp_info.bmiHeader.biSizeImage = 0;
		self.bmp_info = Some(bmp_info);
	}
}

pub struct WindowsWinitSpecifics;

impl WinitPlatformSpecifics for WindowsWinitSpecifics {
	type WindowSpecificData = WindowsWindowSpecificData;

	fn add_window(
		&mut self,
		_window: &WidgetRef<dyn crate::widgets::Window>,
		winit_window: &mut winit::window::Window,
		_event_loop: &mut EventLoop<()>,
	) -> Self::WindowSpecificData {
		let handle = match winit_window.raw_window_handle() {
			RawWindowHandle::Win32(handle) => handle,
			_ => panic!("Windows Platform Specific function called!"),
		};

		let hwnd = windows::Win32::Foundation::HWND(handle.hwnd as isize);

		let mut specific_data = WindowsWindowSpecificData {
			hwnd,
			handle,
			bmp_info_o: None,
			bmp_info: None,
		};

		specific_data.resize_buffer();

		specific_data
	}

	fn remove_window(
		&mut self,
		_window: WidgetRef<crate::platform::winit::Window<Self::WindowSpecificData>>,
	) {
	}

	fn resize_buffer(
		&mut self,
		window: WidgetRef<crate::platform::winit::Window<Self::WindowSpecificData>>,
	) {
		window.get_mut().platform_specific_data.resize_buffer();
	}

	fn flush_window_buffer(
		&mut self,
		window: WidgetRef<crate::platform::winit::Window<Self::WindowSpecificData>>,
	) {
		let mut window = window.get_mut();
		let hwnd = window.platform_specific_data.hwnd;
		let bmp_info = window.platform_specific_data.bmp_info.unwrap();
		unsafe {
			windows::Win32::Graphics::Gdi::InvalidateRect(hwnd, std::ptr::null(), false);
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
				&bmp_info,
				DIB_RGB_COLORS,
			);
			EndPaint(hwnd, &paint_struct);
		}
	}
}

pub fn create_platform() -> Context {
	Context::new(WindowsWinitSpecifics {})
}
