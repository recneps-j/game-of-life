extern crate gl;
extern crate glfw;
extern crate nalgebra_glm as glm;

mod renderer;
mod test;

use glfw::Context;
use renderer::renderer::*;
use std::f32::consts::PI;
use std::ffi::CString;
use std::fs;
use std::ops::Mul;
use std::sync::mpsc::{channel, Receiver, Sender};
use test::Drawable;
use test::Window;

fn error_cb(err: glfw::Error, msg: String, _: &()) {
    println!("GLFW Error: {}, {}", err.to_string(), msg);
}

enum ShaderCommand {
    Zoom(f32),
}

struct Pyramid {
    program: u32,
    width: u32,
    height: u32,
    angle: f32,
    m_projection: glm::Mat4,
    m_view: glm::Mat4,
    m_model: glm::Mat4,
    command_recv: Receiver<ShaderCommand>,
}

impl Drawable for Pyramid {
    fn draw(&mut self, timestamp: std::time::Duration) {
        unsafe {
            if let Ok(cmd) = self.command_recv.try_recv() {
                match cmd {
                    ShaderCommand::Zoom(amount) => {
                        let scale_mat = glm::mat4(
                            (1.0 + amount),
                            0.0,
                            0.0,
                            0.0,
                            0.0,
                            (1.0 + amount),
                            0.0,
                            0.0,
                            0.0,
                            0.0,
                            (1.0 + amount),
                            0.0,
                            0.0,
                            0.0,
                            0.0,
                            1.0,
                        );

                        self.m_view *= scale_mat;
                    }
                    _ => {}
                }
            }

            let rotation_mat = glm::mat4(
                self.angle.cos(),
                0.0,
                self.angle.sin(),
                0.0,
                0.0,
                1.0,
                0.0,
                0.0,
                -self.angle.sin(),
                0.0,
                self.angle.cos(),
                0.0,
                0.0,
                0.0,
                0.0,
                1.0,
            );
            self.m_view *= rotation_mat;
            self.angle += 0.00001;

            let mvp = self.m_projection * self.m_view * self.m_model;
            let transform_uni_name = CString::new("u_transform").unwrap();
            gl::Enable(gl::DEPTH_TEST);
            gl::DepthFunc(gl::LESS);
            gl::UniformMatrix4fv(
                gl::GetUniformLocation(self.program, transform_uni_name.as_ptr()),
                1,
                gl::FALSE,
                mvp.as_ptr() as *const gl::types::GLfloat,
            );
            gl::DrawElements(gl::TRIANGLE_STRIP, 18, gl::UNSIGNED_BYTE, std::ptr::null());
        }
    }
}

fn main() {
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
    glfw.set_error_callback(Some(glfw::Callback {
        f: error_cb,
        data: (),
    }));
    let mut window = Window::new(&mut glfw, 1280, 720).unwrap();

    // Make the window's context current
    window.glfw_window.make_current();
    gl::load_with(|s| glfw.get_proc_address_raw(s));

    window.glfw_window.set_key_polling(true);
    window.glfw_window.set_mouse_button_polling(true);
    window.glfw_window.set_scroll_polling(true);
    window.set_key_event_callback(glfw::Key::Left, |key, code, action, modifiers| {});

    window.set_bg_col((0.2, 0.2, 0.2, 1.0));

    let vtx_path = "shaders/pyramid.vert";
    let vtx = fs::read_to_string(vtx_path)
        .expect(format!("Could not read vertex src from {}", vtx_path).as_str());

    let frag_path = "shaders/pyramid.frag";
    let frag = fs::read_to_string(frag_path)
        .expect(format!("Could not read vertex src frag {}", frag_path).as_str());

    let prog = unsafe { Renderer::compile_shader_from_src(&vtx, &frag).unwrap() };

    unsafe {
        gl::UseProgram(prog);

        let mut array: gl::types::GLuint = 0;
        gl::GenVertexArrays(1, &mut array);
        gl::BindVertexArray(array);

        let mut buffers: [gl::types::GLuint; 2] = [0, 0];
        gl::GenBuffers(2, buffers.as_mut_ptr());

        let indices: Vec<u8> = vec![0, 1, 2, 2, 3, 1, 0, 1, 4, 1, 2, 4, 2, 3, 4, 3, 0, 4];

        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, buffers[0]);
        gl::BufferData(
            gl::ELEMENT_ARRAY_BUFFER,
            (indices.len() * std::mem::size_of::<u8>()) as gl::types::GLsizeiptr,
            indices.as_ptr() as *const gl::types::GLvoid,
            gl::STATIC_DRAW,
        );

        let vertices: Vec<f32> = vec![
            -0.5, -0.5, 0.5, 0.5, -0.5, 0.5, -0.5, -0.5, -0.5, 0.5, -0.5, -0.5, 0.0, 0.5, 0.0,
        ];

        gl::BindBuffer(gl::ARRAY_BUFFER, buffers[1]);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (vertices.len() * std::mem::size_of::<f32>()) as gl::types::GLsizeiptr,
            vertices.as_ptr() as *const gl::types::GLvoid,
            gl::STATIC_DRAW,
        );

        gl::EnableVertexAttribArray(0);
        gl::VertexAttribPointer(
            0,
            3,
            gl::FLOAT,
            gl::FALSE,
            (3 * std::mem::size_of::<f32>()) as gl::types::GLint,
            std::ptr::null(),
        );
    }

    let (sender, receiver) = channel::<ShaderCommand>();
    window.set_scroll_callback(
        Box::new(move |xpos, ypos| {
                sender.send(ShaderCommand::Zoom(ypos as f32 * 0.05)).unwrap();
        }),
    );
    let m_projection: glm::Mat4 = glm::perspective(
        glm::pi::<f32>() / 2.0,
        window.width as f32 / window.height as f32,
        0.1,
        100.0,
    );
    let m_view: glm::Mat4 = glm::look_at(
        &glm::vec3(3.0, 2.0, 2.0), // Camera is at (3, 2, 2), in World Space
        &glm::vec3(0.0, 0.0, 0.0), // and looks at the origin
        &glm::vec3(0.0, 1.0, 0.0), // Head is up (set to 0,-1,0 to look upside-down)
    );
    let m_model = glm::mat4(
        1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
    );
    let pyramid = Box::new(Pyramid {
        program: prog,
        m_projection,
        m_view,
        m_model,
        angle: 0.0,
        width: window.width,
        height: window.height,
        command_recv: receiver,
    });
    window.set_draw_object(pyramid);

    window.run_event_loop(&mut glfw);
}
