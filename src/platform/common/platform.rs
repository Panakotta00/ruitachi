use crate::util::WidgetRef;
use std::cell::RefCell;
use std::rc::Rc;
use winit::event_loop::EventLoop;

pub trait PlatformContext<E> {
	fn new(
		window: &mut winit::window::Window,
		event_loop: &mut EventLoop<E>,
		window_widget: WidgetRef<dyn crate::widgets::Window>,
	) -> Self;
	fn run(&mut self, window: &mut winit::window::Window, event_loop: &mut EventLoop<E>);
}
