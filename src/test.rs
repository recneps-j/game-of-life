extern crate gl;
extern crate glfw;

use std::collections::HashMap;
use std::sync::mpsc::Receiver;
use std::time::Duration;

use glfw::{Action, Context, Key};

pub trait Drawable {
    fn draw(&mut self, timestamp: Duration);
}

pub struct Window {
    width: u32,
    height: u32,
    bg_col: (f32, f32, f32, f32),
    pub glfw_window: glfw::Window,
    pub glfw_events: Receiver<(f64, glfw::WindowEvent)>,

    current_ts: Duration,

    draw_object: Option<Box<dyn Drawable>>,
    key_event_callbacks:
        HashMap<glfw::Key, fn(glfw::Key, glfw::Scancode, glfw::Action, glfw::Modifiers)>,
}

impl Window {
    pub fn new(ctx: &mut glfw::Glfw, width: u32, height: u32) -> Option<Self> {
        ctx.window_hint(glfw::WindowHint::ClientApi(glfw::ClientApiHint::OpenGl));
        ctx.window_hint(glfw::WindowHint::TransparentFramebuffer(true));
        ctx.window_hint(glfw::WindowHint::ContextVersion(3, 2));
        // Create a windowed mode window and its OpenGL context
        let result = ctx.create_window(width, height, "Window", glfw::WindowMode::Windowed);
        match result {
            None => {
                println!("Failed to create GLFW window.");
                return None;
            }
            _ => {}
        }

        let (window, events) = result.unwrap();
        ctx.make_context_current(Some(&window));
        ctx.set_swap_interval(glfw::SwapInterval::Sync(1));
        let w = Window {
            width,
            height,
            bg_col: (0.0, 0.0, 0.0, 0.0),
            glfw_window: window,
            glfw_events: events,
            current_ts: Duration::new(0, 0),
            draw_object: None,
            key_event_callbacks: HashMap::new(),
        };

        Some(w)
    }

    pub unsafe fn draw(&mut self) {
        gl::ClearColor(self.bg_col.0, self.bg_col.1, self.bg_col.2, self.bg_col.3);
        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
    }

    pub fn run_event_loop(&mut self, ctx: &mut glfw::Glfw) {
        // Loop until the user closes the window
        while !self.glfw_window.should_close() {
            self.current_ts += Duration::from_nanos(16667);

            unsafe {
                gl::ClearColor(self.bg_col.0, self.bg_col.1, self.bg_col.2, self.bg_col.3);
                gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            }

            match &mut self.draw_object {
                Some(cb) => cb.draw(self.current_ts),
                None => {}
            }

            // Swap front and back buffers
            self.glfw_window.swap_buffers();

            // Poll for and process events
            ctx.poll_events();

            for (_, event) in glfw::flush_messages(&self.glfw_events) {
                println!("{:?}", event);
                match event {
                    glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                        self.glfw_window.set_should_close(true)
                    }
                    glfw::WindowEvent::Key(key, scode, action, modifiers) => {
                        match self.key_event_callbacks.get(&key) {
                            Some(f) => f(key, scode, action, modifiers),
                            None => {}
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    pub fn set_bg_col(&mut self, col: (f32, f32, f32, f32)) {
        self.bg_col = col;
    }

    pub fn set_draw_object(&mut self, draw_object: Box<dyn Drawable>) {
        self.draw_object = Some(draw_object);
    }

    pub fn set_key_event_callback(
        &mut self,
        event_type: glfw::Key,
        callback: fn(glfw::Key, glfw::Scancode, glfw::Action, glfw::Modifiers),
    ) {
        self.key_event_callbacks.insert(event_type, callback);
    }
}
