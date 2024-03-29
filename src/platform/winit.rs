use std::cell::RefCell;
use crate::{events, events::EventContext, platform::common::PlatformContext, util::*};
use cgmath::Vector2;

use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use winit::{
	dpi::PhysicalSize,
	event::{ElementState, Event, WindowEvent},
	event_loop::{ControlFlow, EventLoop},
	platform::run_return::EventLoopExtRunReturn,
};
use winit::event_loop::EventLoopWindowTarget;
use crate::application::GUIApplication;
use crate::platform::common::PlatformMessage;

pub type WindowId = winit::window::WindowId;

/// Implementation of platform specific code that is not handled by winit
/// like more specific I/O or draw buffer creation and management.
pub trait WinitPlatformSpecifics {
	type WindowSpecificData;

	fn add_window(
		&mut self,
		window: &WidgetRef<dyn crate::widgets::Window>,
		winit_window: &mut winit::window::Window,
		event_loop: &mut EventLoop<()>,
	) -> Self::WindowSpecificData;
	fn remove_window(&mut self, window: SharedRef<Window<Self::WindowSpecificData>>);
	fn resize_buffer(&mut self, window: SharedRef<Window<Self::WindowSpecificData>>);
	fn flush_window_buffer(&mut self, window: SharedRef<Window<Self::WindowSpecificData>>);
}

/// Holds information specific to a single window in the context of winit.
/// Additionally contains a reference to the frameworks window representation
/// and platform specific data like draw buffers.
pub struct Window<T> {
	pub winit_window: winit::window::Window,
	pub framework_window: WidgetRef<dyn crate::widgets::Window>,
	pub platform_specific_data: T,
	pub skia_data: Option<(skia_safe::ImageInfo, skia_safe::Bitmap)>,
	pub size: PhysicalSize<u32>,
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
	windows: HashMap<WindowId, SharedRef<Window<PS::WindowSpecificData>>>,
	deferred_messages: RefCell<Vec<PlatformMessage>>,

	last_cursor_pos: Vector2<scalar>,
}

impl<PS> Context<PS>
where
	PS: WinitPlatformSpecifics,
{
	/// Creates a new winit platform context with the given platform specifics
	pub fn new(platform_specifics: PS) -> Context<PS> {
		Self {
			platform_specifics,
			windows: Default::default(),
			deferred_messages: Default::default(),
			last_cursor_pos: Vector2::new(0.0, 0.0),
		}
	}

	fn window_by_id(&self, id: WindowId) -> Option<SharedRef<Window<PS::WindowSpecificData>>> {
		self.windows.get(&id).map(|w| w.clone())
	}

	fn resize_buffer(&mut self, window: SharedRef<Window<PS::WindowSpecificData>>) {
		let size;
		{
			let mut window = window.get_mut();
			size = window.winit_window.inner_size();

			let old_size = window.size;
			if old_size == size {
				return;
			}
			window.size = size;

			let info = skia_safe::ImageInfo::new(
				(size.width as i32, size.height as i32),
				skia_safe::ColorType::BGRA8888,
				skia_safe::AlphaType::Unpremul,
				None,
			);
			if let Some((skia_info, skia_bitmap)) = &mut window.skia_data {
				*skia_info = info;
				skia_bitmap.reset();
			} else {
				window.skia_data = Some((info, skia_safe::Bitmap::new()));
			}
			let skia_data = window.skia_data.as_mut().unwrap();
			skia_data.1.alloc_pixels_flags(&skia_data.0);
			drop(window);
		}
		self.platform_specifics.resize_buffer(window.clone());

		let geometry = Geometry::new(
			Vector2::new(0.0, 0.0),
			Vector2::new(size.width as scalar, size.height as scalar),
			Vector2::new(0.0, 0.0),
			Vector2::new(1.0, 1.0),
		);
		window.get().framework_window.get().deref().arrange_children(geometry);
	}

	fn handle_window_event(this: &RefCell<Self>, event: WindowEvent, window: SharedRef<Window<PS::WindowSpecificData>>, window_target: &EventLoopWindowTarget<()>, control_flow: &mut ControlFlow, event_context: &RefCell<EventContext>) {
		match event {
			WindowEvent::CloseRequested => {
				GUIApplication::get().remove_window(window.get().framework_window.clone());
			}
			WindowEvent::Resized(_size) => {
				window.get().winit_window.request_redraw();
			}
			WindowEvent::CursorMoved {
					device_id: _,
					position,
					modifiers: _,
			} => {
				let pos = Vector2::new(position.x as f32, position.y as f32);
				this.borrow_mut().last_cursor_pos = pos;

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
				event_context.borrow_mut().handle_mouse_move(&path, 0, &this.borrow().last_cursor_pos);

				window.get().winit_window.request_redraw();
			}
			WindowEvent::MouseInput {
				device_id: _,
				button,
				state,
				modifiers: _,
			} => {
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
						&this.borrow().last_cursor_pos,
					);
					let pos = this.borrow().last_cursor_pos.clone();
					match state {
						ElementState::Pressed => event_context.borrow_mut()
							.handle_mouse_button_down(
								&path,
								0,
								conv_mouse_button(button),
								&pos,
							),
						ElementState::Released => event_context.borrow_mut()
							.handle_mouse_button_up(
								&path,
								0,
								conv_mouse_button(button),
								&pos,
							),
					}

					window.get().winit_window.request_redraw();
				}
			}
			WindowEvent::KeyboardInput {
				device_id: _,
				input,
				is_synthetic: _,
			} => {
				match input.state {
					// TODO: Add multi device support
					ElementState::Pressed => {
						event_context.borrow_mut().handle_key_down(
							0,
							input.scancode as usize,
							input.virtual_keycode,
						);
					}
					ElementState::Released => {
						event_context.borrow_mut().handle_key_up(
							0,
							input.scancode as usize,
							input.virtual_keycode,
						);
					}
				}
				window.get().winit_window.request_redraw();
			}
			WindowEvent::ReceivedCharacter(char) => {
				// TODO: Add multi device support
				event_context.borrow_mut().handle_text(0, char);
				window.get().winit_window.request_redraw();
			}
			WindowEvent::CursorLeft { device_id: _ } => {
				// TODO: Add multi device support
				event_context.borrow_mut().handle_cursor_leave(0);
				window.get().winit_window.request_redraw();
			}
			WindowEvent::CursorEntered { device_id: _ } => {
				// TODO: Add multi device support
				event_context.borrow_mut().handle_cursor_enter(0);
				window.get().winit_window.request_redraw();
			}
			_ => (),
		}
	}

	fn handle_event(this: &RefCell<Self>, event: Event<()>, window_target: &EventLoopWindowTarget<()>, control_flow: &mut ControlFlow, event_context: &RefCell<EventContext>) {
		*control_flow = ControlFlow::Wait;

		match event {
			Event::WindowEvent {
				event,
				window_id
			} => {
				let window = this.borrow().window_by_id(window_id);
				if let Some(window) = window {
					Self::handle_window_event(this, event, window, window_target, control_flow, event_context);
				}
			}
			Event::RedrawRequested(window_id) => {
				let window = this.borrow().window_by_id(window_id);
				if let Some(window) = window {
					this.borrow_mut().resize_buffer(window.clone());

					let size = window.get().winit_window.inner_size();
					let window_ref = window.get();

					let mut canvas = skia_safe::Canvas::from_bitmap(
						&window_ref.skia_data.as_ref().unwrap().1,
						None,
					);

					window_ref.framework_window.get().draw(
						&mut canvas,
						(size.width as scalar, size.height as scalar),
					);

					drop(window_ref);

					this.borrow_mut().platform_specifics.flush_window_buffer(window.clone());
				}
			}
			Event::RedrawEventsCleared => {
				/*self.wayland_event_queue
				.dispatch_pending(&mut (), || {

				})
				.expect("failed to dispatch wayland event queue");*/
			}
			_ => ()
		}

		// End Event loop if no windows are left or if there are messages to handle
		if this.borrow().windows.len() < 1 || this.borrow().deferred_messages.borrow().len() > 0 {
			*control_flow = ControlFlow::Exit;
		}
	}
}

impl<PS> PlatformContext for Context<PS>
where
	PS: WinitPlatformSpecifics,
{
	fn message(&self, message: PlatformMessage) {
		match message {
			_ => self.deferred_messages.borrow_mut().push(message),
		}
	}

	fn run(this: &RefCell<Self>, event_context: &RefCell<EventContext>) {
		let mut event_loop = EventLoop::new();
		while {this.borrow().windows.len() > 0 || this.borrow().deferred_messages.borrow().len() > 0} {
			event_loop.run_return(|event, window_target, control_flow| {
				Self::handle_event(this, event, window_target, control_flow, event_context);
			});

			let mut messages: Vec<_> = std::mem::take(this.borrow_mut().deferred_messages.borrow_mut().as_mut());
			for message in messages {
				match message {
					PlatformMessage::NewWindow(window) => {
						let mut winit_window = winit::window::WindowBuilder::new()
							.with_title("Hello World")
							.with_decorations(true)
							.with_transparent(true)
							.build(&event_loop)
							.unwrap();

						let id: WindowId = winit_window.id();

						window.get_mut().set_id(Some(id));

						let platform_specific_data = this.borrow_mut().platform_specifics.add_window(
							&window,
							&mut winit_window,
							&mut event_loop,
						);

						let mut window = SharedRef::new(Window {
							winit_window,
							framework_window: window.clone(),
							platform_specific_data,
							skia_data: None,
							size: Default::default(),
						});

						this.borrow_mut().resize_buffer(window.clone());

						this.borrow_mut().windows.insert(id, window);
					}
					PlatformMessage::RemoveWindow(window) => {
						let id = match window.get().id() {
							Some(id) => id,
							None => return,
						};
						let window = match this.borrow_mut().windows.remove(&id) {
							Some(w) => w,
							None => return,
						};
						this.borrow_mut().platform_specifics.remove_window(window);
					}
				}
			}
		}
	}

	fn set_capture_cursor(&mut self, _cursor: usize, _should_capture: bool) {}
}

pub fn conv_mouse_button(btn: winit::event::MouseButton) -> events::input::MouseButton {
	match btn {
		winit::event::MouseButton::Left => events::input::MouseButton::Left,
		winit::event::MouseButton::Right => events::input::MouseButton::Right,
		winit::event::MouseButton::Middle => events::input::MouseButton::Middle,
		winit::event::MouseButton::Other(c) => events::input::MouseButton::Other(c),
	}
}
