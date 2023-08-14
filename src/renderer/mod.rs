extern crate gl;

pub mod renderer {
    use std::ffi::{CStr, CString};

    pub struct Renderer {
        shader_programs: Vec<(String, u32)>,
    }

    impl Renderer {
        pub fn new() -> Self {
            Renderer {
                shader_programs: Vec::new(),
            }
        }

        fn add_shader_program(
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
                .push((program_name.clone(), shader_handle));
            Ok(())
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
}
