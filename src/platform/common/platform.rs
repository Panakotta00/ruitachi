use winit::window::Window;

pub trait Platform {
    fn IntializeWindow(window : &mut Window);
    fn DrawWindow(window : &mut Window, func : fn(window : &mut Window, painter : &mut crate::paint::Painter));
}
