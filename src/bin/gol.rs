use engine2d::{Drawable, RenderCtx, Window};

struct Gol {

}

impl Drawable for Gol {
    fn draw(&mut self, timestamp: std::time::Duration) {
        println!("Drawing at: {} ms", timestamp.as_millis());
    }
}

fn main() {
    let mut ctx = RenderCtx::create();
    let mut win = Window::new(&mut ctx, 1920, 1080).unwrap();
    win.set_bg_col((0.2, 0.2, 0.2, 1.0));

    let mut gol = Gol{};
    win.set_draw_object(Box::new(gol));

    win.run_event_loop(&mut ctx);
}
