extern crate gl;
extern crate glfw;

use std::collections::HashMap;
use std::sync::mpsc::Receiver;
use std::time::Duration;
use std::ffi::CString;
use glfw::{Action, Context, Key};

pub trait Drawable {
    fn draw(&mut self, timestamp: Duration);
}

pub struct RenderCtx {
    glfw_ctx: glfw::Glfw,
    shader_programs: HashMap<String, u32>,
}

impl RenderCtx {
    pub fn create() -> Self {
        let mut glfw_ctx = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
        glfw_ctx.set_error_callback(Some(glfw::Callback {
            f: Self::error_cb,
            data: (),
        }));
        return RenderCtx {
            glfw_ctx,
            shader_programs: HashMap::new(),
        };
    }

    fn error_cb(err: glfw::Error, msg: String, _: &()) {
        println!("GLFW Error: {}, {}", err.to_string(), msg);
    }

    pub fn add_shader_program(
        &mut self,
        program_name: &String,
        shader_handle: u32,
    ) -> Result<(), &str> {
        for entry in self.shader_programs.iter() {
            if entry.0.contains(program_name) {
                return Err("A program with that name already exists");
            }
        }

        self.shader_programs
            .insert(program_name.clone(), shader_handle);
        Ok(())
    }

    pub fn get_shader_program(&mut self, program_name: &String) -> Option<&u32> {
        self.shader_programs.get(program_name)
    }

    pub unsafe fn compile_shader_from_src(
        vertex_src: &String,
        fragment_src: &String,
    ) -> Option<u32> {
        let vertex_src_cstr = CString::new(vertex_src.as_str()).unwrap();
        let fragment_src_cstr = CString::new(fragment_src.as_str()).unwrap();

        let vertex_shader = gl::CreateShader(gl::VERTEX_SHADER);
        gl::ShaderSource(
            vertex_shader,
            1,
            &vertex_src_cstr.as_c_str().as_ptr(),
            std::ptr::null(),
        );
        gl::CompileShader(vertex_shader);
        let mut success: gl::types::GLint = 1;
        gl::GetShaderiv(vertex_shader, gl::COMPILE_STATUS, &mut success);
        if success == 0 {
            gl::DeleteShader(vertex_shader);
            return None;
        }

        let fragment_shader = gl::CreateShader(gl::FRAGMENT_SHADER);
        gl::ShaderSource(
            fragment_shader,
            1,
            &fragment_src_cstr.as_c_str().as_ptr(),
            std::ptr::null(),
        );
        gl::CompileShader(fragment_shader);
        gl::GetShaderiv(fragment_shader, gl::COMPILE_STATUS, &mut success);

        if success == 0 {
            gl::DeleteShader(vertex_shader);
            gl::DeleteShader(fragment_shader);
            return None;
        }

        let shader_program = gl::CreateProgram();
        gl::AttachShader(shader_program, vertex_shader);
        gl::AttachShader(shader_program, fragment_shader);
        gl::LinkProgram(shader_program);
        gl::GetShaderiv(shader_program, gl::LINK_STATUS, &mut success);

        if success == 0 {
            gl::DetachShader(shader_program, vertex_shader);
            gl::DetachShader(shader_program, fragment_shader);
            gl::DeleteShader(vertex_shader);
            gl::DeleteShader(fragment_shader);
            gl::DeleteProgram(shader_program);
            return None;
        }

        gl::DetachShader(shader_program, vertex_shader);
        gl::DetachShader(shader_program, fragment_shader);
        gl::DeleteShader(vertex_shader);
        gl::DeleteShader(fragment_shader);

        Some(shader_program)
    }
}

pub struct Window {
    pub width: u32,
    pub height: u32,
    bg_col: (f32, f32, f32, f32),
    pub glfw_window: glfw::Window,
    pub glfw_events: Receiver<(f64, glfw::WindowEvent)>,

    current_ts: Duration,

    draw_object: Option<Box<dyn Drawable>>,
    key_event_callbacks:
        HashMap<glfw::Key, fn(glfw::Key, glfw::Scancode, glfw::Action, glfw::Modifiers)>,
    mouse_button_callbacks: HashMap<
        glfw::MouseButton,
        Box<dyn FnMut(glfw::MouseButton, glfw::Action, glfw::Modifiers)>,
    >,
    scroll_callback: Option<Box<dyn FnMut(f64, f64)>>,
}

impl Window {
    pub fn new(ctx: &mut RenderCtx, width: u32, height: u32) -> Option<Self> {
        ctx.glfw_ctx
            .window_hint(glfw::WindowHint::ClientApi(glfw::ClientApiHint::OpenGl));
        ctx.glfw_ctx
            .window_hint(glfw::WindowHint::TransparentFramebuffer(true));
        ctx.glfw_ctx
            .window_hint(glfw::WindowHint::ContextVersion(3, 2));
        // Create a windowed mode window and its OpenGL context
        let result =
            ctx.glfw_ctx
                .create_window(width, height, "Window", glfw::WindowMode::Windowed);
        match result {
            None => {
                println!("Failed to create GLFW window.");
                return None;
            }
            _ => {}
        }

        let (window, events) = result.unwrap();
        ctx.glfw_ctx.make_context_current(Some(&window));
        ctx.glfw_ctx.set_swap_interval(glfw::SwapInterval::Sync(1));
        let mut w = Window {
            width,
            height,
            bg_col: (0.0, 0.0, 0.0, 0.0),
            glfw_window: window,
            glfw_events: events,
            current_ts: Duration::new(0, 0),
            draw_object: None,
            key_event_callbacks: HashMap::new(),
            mouse_button_callbacks: HashMap::new(),
            scroll_callback: None,
        };

        w.glfw_window.make_current();
        gl::load_with(|s| ctx.glfw_ctx.get_proc_address_raw(s));

        Some(w)
    }

    pub fn run_event_loop(&mut self, ctx: &mut RenderCtx) {
        // Loop until the user closes the window
        while !self.glfw_window.should_close() {
            self.current_ts += Duration::from_nanos(16666667);

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
            ctx.glfw_ctx.poll_events();

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
                    glfw::WindowEvent::MouseButton(mouse_button, action, modifiers) => {
                        match self.mouse_button_callbacks.get_mut(&mouse_button) {
                            Some(f) => f(mouse_button, action, modifiers),
                            None => {}
                        }
                    }
                    glfw::WindowEvent::Scroll(xpos, ypos) => match &mut self.scroll_callback {
                        Some(f) => f(xpos, ypos),
                        None => {}
                    },
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

    pub fn set_mouse_button_callback(
        &mut self,
        event_type: glfw::MouseButton,
        callback: Box<dyn FnMut(glfw::MouseButton, glfw::Action, glfw::Modifiers)>,
    ) {
        self.mouse_button_callbacks.insert(event_type, callback);
    }

    pub fn set_scroll_callback(&mut self, callback: Box<dyn FnMut(f64, f64)>) {
        self.scroll_callback = Some(callback);
    }
}
