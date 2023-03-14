use std::borrow::Borrow;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use cgmath::Vector2;
use skia_safe::ImageInfo;
use skia_safe::wrapper::NativeTransmutableWrapper;
use winit::event::{ElementState, Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::platform::run_return::EventLoopExtRunReturn;
use winit::platform::windows::{WindowBuilderExtWindows, WindowExtWindows};
use crate::events;
use crate::events::EventContext;
use crate::platform::common::PlatformContext;
use crate::util::*;

pub type WindowId = winit::window::WindowId;

/// Implementation of platform specific code that is not handled by winit
/// like more specific I/O or draw buffer creation and management.
pub trait WinitPlatformSpecifics {
	type WindowSpecificData;

	fn add_window(&mut self, window: &WidgetRef<dyn crate::widgets::Window>, winit_window: &mut winit::window::Window) -> Self::WindowSpecificData;
	fn remove_window(&mut self, window: WidgetRef<Window<Self::WindowSpecificData>>);
	fn resize_window(&mut self, window: WidgetRef<Window<Self::WindowSpecificData>>);
	fn flush_window_buffer(&mut self, window: WidgetRef<Window<Self::WindowSpecificData>>);
}

/// Holds information specific to a single window in the context of winit.
/// Additionally contains a reference to the frameworks window representation
/// and platform specific data like draw buffers.
pub struct Window<T> {
	pub winit_window: winit::window::Window,
	pub framework_window: WidgetRef<dyn crate::widgets::Window>,
	pub platform_specific_data: T,
	pub skia_data: Option<(skia_safe::ImageInfo, skia_safe::Bitmap)>,
}

impl<T> Window<T> {
	pub fn resize_buffer(&mut self) {
		let size = self.winit_window.inner_size();
		let info = skia_safe::ImageInfo::new(
			(size.width as i32, size.height as i32),
			skia_safe::ColorType::BGRA8888,
			skia_safe::AlphaType::Unpremul,
			None,
		);
		if let Some((skia_info, skia_bitmap)) = &mut self.skia_data {
			*skia_info = info;
			skia_bitmap.reset();
		} else {
			self.skia_data = Some((
				info,
				skia_safe::Bitmap::new(),
			));
		}
		let skia_data = self.skia_data.as_mut().unwrap();
		skia_data.1.alloc_pixels_flags(&skia_data.0);
	}
}

/// Fully implemented platform context for winit.
/// Requires Platform Specific implementation for draw buffer creating and handling
/// as well as more specialized I/O not covered by winit.
///
/// # Arguments
///
/// * `PS` - Type of the platform specific implementations needed
pub struct Context<PS: WinitPlatformSpecifics> {
	platform_specifics: PS,
	windows: HashMap<WindowId, WidgetRef<Window<PS::WindowSpecificData>>>,
	pub event_loop: Option<EventLoop<()>>,
	deferred_widget_creating: Vec<WidgetRef<dyn crate::widgets::Window>>,

	last_cursor_pos: Vector2<scalar>,
}

impl<PS> Context<PS> where PS: WinitPlatformSpecifics {
	/// Creates a new winit platform context with the given platform specifics
	pub fn new(platform_specifics: PS) -> Context<PS> {
		let event_loop = EventLoop::new();

		Self {
			platform_specifics,
			windows: Default::default(),
			event_loop: Some(event_loop),
			deferred_widget_creating: Default::default(),
			last_cursor_pos: Vector2::new(0.0, 0.0),
		}
	}

	fn window_by_id(&mut self, id: WindowId) -> Option<WidgetRef<Window<PS::WindowSpecificData>>> {
		self.windows.get(&id).map(|w| w.clone())
	}
}

impl<PS> PlatformContext for Context<PS> where PS: WinitPlatformSpecifics {
	fn add_window(&mut self, window: &WidgetRef<dyn crate::widgets::Window>) {
		self.deferred_widget_creating.push(window.clone());
	}

	fn remove_window(&mut self, window: &WidgetRef<dyn crate::widgets::Window>) -> bool {
		let id = match window.get().id() {
			Some(id) => id,
			None => return false,
		};
		let mut window = match self.windows.remove(&id) {
			Some(w) => w,
			None => return false,
		};

		self.platform_specifics.remove_window(window);

		true
	}

	fn run(&mut self, event_context: &mut EventContext) {
		loop {
			let mut event_loop = self.event_loop.take();
			let mut loop_braked = false;
			event_loop.as_mut().unwrap().run_return(|event, _, control_flow| {
				*control_flow = ControlFlow::Wait;

				match event {
					Event::WindowEvent {
						event: WindowEvent::CloseRequested,
						window_id,
					} => if let Some(window) = self.window_by_id(window_id) {
						self.remove_window(&window.get().framework_window);
						if self.windows.len() < 1 {
							*control_flow = ControlFlow::Exit;
						}
					}
					Event::WindowEvent {
						event: WindowEvent::Resized(size),
						window_id,
					} => if let Some(window) = self.window_by_id(window_id) {
						window.get().resize_buffer();
						self.platform_specifics.resize_window(window);
					}
					Event::RedrawRequested(window_id)
					=> if let Some(window) = self.window_by_id(window_id) {
						let size = window.get().winit_window.inner_size();
						let window_ref = window.get();

						let mut canvas = skia_safe::Canvas::from_bitmap(&window_ref.skia_data.as_ref().unwrap().1, None);

						window_ref.framework_window.get().draw(&mut canvas, (size.width as scalar, size.height as scalar));

						drop(window_ref);

						self.platform_specifics.flush_window_buffer(window.clone());
					}
					Event::WindowEvent {
						event:
						WindowEvent::CursorMoved {
							device_id,
							position,
							modifiers,
						},
						window_id,
					} => {
						let pos = Vector2::new(position.x as f32, position.y as f32);
						self.last_cursor_pos = pos;
						if let Some(window) = self.window_by_id(window_id) {
							let size = window.get().winit_window.inner_size();
							let geometry = Geometry::new(
								Vector2::new(0.0, 0.0),
								Vector2::new(size.width as scalar, size.height as scalar),
								Vector2::new(0.0, 0.0),
								Vector2::new(1.0, 1.0),
							);

							// TODO: Add multi device support
							let path = events::get_widget_path_under_position(
								geometry,
								window.get().framework_window.clone(),
								&pos,
							);
							event_context.handle_mouse_move(&path, 0, &self.last_cursor_pos);

							window.get().winit_window.request_redraw();
						}
					}
					Event::WindowEvent {
						event:
						WindowEvent::MouseInput {
							device_id,
							button,
							state,
							modifiers,
						},
						window_id,
					} => if let Some(window) = self.window_by_id(window_id) {
						if button == winit::event::MouseButton::Left {
							let size = window.get().winit_window.inner_size();
							let geometry = Geometry::new(
								Vector2::new(0.0, 0.0),
								Vector2::new(size.width as scalar, size.height as scalar),
								Vector2::new(0.0, 0.0),
								Vector2::new(1.0, 1.0),
							);

							let path = events::get_widget_path_under_position(
								geometry,
								window.get().framework_window.clone(),
								&self.last_cursor_pos,
							);
							let pos = self.last_cursor_pos.clone();
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

							window.get().winit_window.request_redraw();
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
					} => if let Some(window) = self.window_by_id(window_id) {
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
						window.get().winit_window.request_redraw();
					}
					Event::WindowEvent {
						window_id,
						event: WindowEvent::ReceivedCharacter(char),
					} => if let Some(window) = self.window_by_id(window_id) {
						// TODO: Add multi device support
						event_context.handle_text(0, char);
						window.get().winit_window.request_redraw();
					}
					Event::WindowEvent {
						window_id,
						event: WindowEvent::CursorLeft { device_id },
					} => if let Some(window) = self.window_by_id(window_id) {
						// TODO: Add multi device support
						event_context.handle_cursor_leave(0);
						window.get().winit_window.request_redraw();
					}
					Event::RedrawEventsCleared => {
						/*self.wayland_event_queue
					.dispatch_pending(&mut (), || {

					})
					.expect("failed to dispatch wayland event queue");*/
					}
					_ => (),
				}

				if self.deferred_widget_creating.len() > 0 {
					*control_flow = ControlFlow::Exit;
					loop_braked = true;
				}
			});
			self.event_loop = event_loop;

			if loop_braked {
				for window in self.deferred_widget_creating.drain(0..self.deferred_widget_creating.len()) {
					let mut winit_window = winit::window::WindowBuilder::new()
						.with_title("Hello World")
						.with_decorations(true)
						.with_transparent(true)
						.build(self.event_loop.as_ref().unwrap())
						.unwrap();

					let id: WindowId = winit_window.id();

					window.get().set_id(Some(id));

					let platform_specific_data = self.platform_specifics.add_window(&window, &mut winit_window);

					let mut window = Window{
						winit_window,
						framework_window: window.clone(),
						platform_specific_data,
						skia_data: None,
					};

					window.resize_buffer();

					self.windows.insert(id, WidgetRef::new(window));
				}
			} else {
				break
			}
		}
	}

	fn set_capture_cursor(&mut self, cursor: usize, should_capture: bool) {}
}

pub fn conv_mouse_button(btn: winit::event::MouseButton) -> events::input::MouseButton {
	match btn {
		winit::event::MouseButton::Left => events::input::MouseButton::Left,
		winit::event::MouseButton::Right => events::input::MouseButton::Right,
		winit::event::MouseButton::Middle => events::input::MouseButton::Middle,
		winit::event::MouseButton::Other(c) => events::input::MouseButton::Other(c),
	}
}