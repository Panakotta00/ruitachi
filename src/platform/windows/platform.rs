use std::cmp::max;
use std::ffi::c_void;
use raw_window_handle::{HasRawWindowHandle, RawWindowHandle, Win32Handle};
use windows::Win32::Graphics::Gdi::{BeginPaint, BI_RGB, BITMAPINFO, BITMAPINFOHEADER, DIB_RGB_COLORS, PAINTSTRUCT, RGBQUAD, SRCCOPY};
use winit::window::Window;

pub struct Platform {}

impl crate::platform::common::Platform for Platform {
    fn IntializeWindow(window: &mut Window) {

    }

    fn DrawWindow(window: &mut Window, func: fn(&mut Window, &mut crate::paint::Painter)) {
        let handle = match window.raw_window_handle() {
            RawWindowHandle::Win32(handle) => { handle }
            _ => panic!("Windows Platform Specific function called!")
        };

        let hwnd = windows::Win32::Foundation::HWND(handle.hwnd as isize);
        unsafe { windows::Win32::Graphics::Gdi::InvalidateRect(hwnd, std::ptr::null(), false); }
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
        let mut pixels2 = unsafe { (*bmpInfo).bmiColors.as_mut_ptr() as *mut u8 };
        let pixels_p = unsafe { std::slice::from_raw_parts_mut(pixels2, (bmpw * bmph) as usize * std::mem::size_of::<RGBQUAD>()) };

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

        func(window, &mut surface);

        unsafe {
            windows::Win32::Graphics::Gdi::StretchDIBits(hdc, 0, 0, bmpw, bmph, 0, 0, bmpw, bmph, pixels.as_ptr() as *const c_void, bmpInfo, DIB_RGB_COLORS, SRCCOPY);

            windows::Win32::Graphics::Gdi::EndPaint(hwnd, &paintStruct);
        }
    }
}