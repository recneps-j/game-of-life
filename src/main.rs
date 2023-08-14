extern crate gl;
extern crate glfw;

mod renderer;
mod test;

use glfw::Context;
use renderer::renderer::*;
use std::ffi::{CStr, CString};
use test::Drawable;
use test::Window;

fn error_cb(err: glfw::Error, msg: String, _: &()) {
    println!("GLFW Error: {}, {}", err.to_string(), msg);
}

struct Pyramid {
    program: u32,
    m_transform: Vec<f32>,
}

impl Drawable for Pyramid {
    fn draw(&mut self, timestamp: std::time::Duration) {
        unsafe {
            let transform_uni_name = CString::new("u_transform").unwrap();
            gl::UniformMatrix4fv(
                gl::GetUniformLocation(self.program, transform_uni_name.as_ptr()),
                1,
                gl::FALSE,
                self.m_transform.as_ptr() as *const gl::types::GLfloat,
            );
            gl::DrawArrays(
                gl::TRIANGLES, // mode
                0,             // starting index in the enabled arrays
                5,             // number of indices to be rendered
            );
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
    window.set_key_event_callback(glfw::Key::Left, |Key, Code, Action, Modifier| {
        println!("Got to my custom callback");
    });

    window.set_bg_col((0.2, 0.2, 0.2, 1.0));

    let vtx = String::from(
"#version 330 core\n
\n
layout (location = 0) in vec3 Position;\n
out vec4 v_colour;\n
\n
uniform mat4 u_transform;\n
\n
void main()\n
{\n
    vec4 fg_col = vec4(1.0f, 0.5f, 0.2f, 1.0f);\n
    vec4 bg_col = vec4(0.4f, 0.2f, 0.1f, 1.0f);\n
    v_colour = mix(fg_col, bg_col, Position.z * 2.0);\n
    gl_Position = u_transform * vec4(Position, 1.0);\n
}\n",
    );
    let frag = String::from(
"#version 330 core\n
\n
out vec4 Color;\n
in vec4 v_colour;\n
\n
void main()\n
{\n
Color = v_colour;\n
}",
    );
    let prog = unsafe { Renderer::compile_shader_from_src(&vtx, &frag).unwrap() };
    unsafe {
        gl::UseProgram(prog);

        let mut array: gl::types::GLuint = 0;
        gl::GenVertexArrays(1, &mut array);
        gl::BindVertexArray(array);

        let mut buffers: [gl::types::GLuint; 2] = [0, 0];
        gl::GenBuffers(2, buffers.as_mut_ptr());

        let vertices: Vec<f32> = vec![-0.5, -0.5, 0.5,
                                       0.5, -0.5, 0.5,
                                      -0.5, -0.5,-0.5,
                                       0.5, -0.5,-0.5,
                                       0.0,  0.5, 0.0];

        gl::BindBuffer(gl::ARRAY_BUFFER, buffers[0]);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (vertices.len() * std::mem::size_of::<f32>()) as gl::types::GLsizeiptr,
            vertices.as_ptr() as *const gl::types::GLvoid,
            gl::STATIC_DRAW,
        );

        gl::EnableVertexAttribArray(0); // this is "layout (location = 0)" in vertex shader
        gl::VertexAttribPointer(
            0,         // index of the generic vertex attribute ("layout (location = 0)")
            3,         // the number of components per generic vertex attribute
            gl::FLOAT, // data type
            gl::FALSE, // normalized (int-to-float conversion)
            (3 * std::mem::size_of::<f32>()) as gl::types::GLint, // stride (byte offset between consecutive attributes)
            std::ptr::null(),                                     // offset of the first component
        );
    }

    let m_transform: Vec<f32> = vec![
        1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
    ];
    let pyramid = Box::new(Pyramid {
        program: prog,
        m_transform,
    });
    window.set_draw_object(pyramid);

    window.run_event_loop(&mut glfw);
}
