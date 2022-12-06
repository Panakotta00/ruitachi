use winit::event_loop::EventLoop;
use winit::window::Window;

pub trait PlatformContext<E> {
    fn new(window: &mut Window, event_loop: &mut EventLoop<E>) -> Self;
    fn run(&mut self, window: &mut Window, event_loop: &mut EventLoop<E>);
    fn draw_window(&mut self, window : &mut Window, func : fn(window : &mut Window, painter : &mut crate::paint::Painter));
}
