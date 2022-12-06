use rand::Rng;

pub trait Window {
    fn draw(&mut self, painter: &mut crate::paint::Painter);
}

pub struct WindowImpl {

}

impl Window for WindowImpl {
    fn draw(&mut self, painter: &mut crate::paint::Painter) {
        let mut rnd = rand::thread_rng();
        let canvas = painter.canvas();
        canvas.clear(skia_safe::Color::DARK_GRAY);
        let mut paint = skia_safe::Paint::default();
        paint.set_anti_alias(true);
        //paint.set_color(skia_safe::Color::new(rnd.gen_range(0..0xFFFFFF) << 8 | 0xFF));
        paint.set_color(skia_safe::Color::BLUE);
        canvas.draw_circle((100, 100), 90.0, &paint);
    }
}