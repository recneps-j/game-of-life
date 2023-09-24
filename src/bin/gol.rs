use engine2d::{Drawable, RenderCtx, Window};
use gl;
use std::fs;
use std::time::Duration;

struct Gol {
    speed: u32,
    cells: Vec<[u8; 64]>,
    cells_buffer: Vec<f32>,
    gl_array: gl::types::GLuint,
    gl_buffer: gl::types::GLuint,
    frame_counter: usize,
    period_frames: usize,
}

impl Gol {
    fn new() -> Self {
        let mut gol = Gol {
            speed: 1,
            cells: vec![[0 as u8; 64]; 64],
            cells_buffer: Vec::new(),
            gl_array: 0,
            gl_buffer: 0,
            frame_counter: 0,
            period_frames: 0,
        };
        gol.period_frames = 60 * gol.speed as usize;
        gol
    }

    fn update_cells(&mut self) {
        for i in 0..64 {
            for j in 0..64 {
                let mut count = 0;
                if i > 0 {
                    if j > 0 {
                        count += self.cells[i - 1][j - 1];
                    }
                    count += self.cells[i - 1][j];
                    if j < 63 {
                        count += self.cells[i - 1][j + 1];
                    }
                }
                if i < 63 {
                    if j > 0 {
                        count += self.cells[i + 1][j - 1];
                    }
                    count += self.cells[i + 1][j];
                    if j < 63 {
                        count += self.cells[i + 1][j + 1];
                    }
                }
                if j > 0 {
                    count += self.cells[i][j - 1];
                }
                if j < 63 {
                    count += self.cells[i][j + 1];
                }

                if self.cells[i][j] > 0 {
                    if count < 2 || count > 3 {
                        self.cells[i][j] = 0;
                    }
                } else {
                    if count == 3 {
                        self.cells[i][j] = 1;
                    }
                }
            }

            let mut vertices = Vec::<f32>::new();
            for i in 0..self.cells.len() {
                for j in 0..self.cells[0].len() {
                    if self.cells[i][j] > 0 {
                        vertices.push(-1.0 + 2.0 / 64.0 * i as f32);
                        vertices.push(-1.0 + 2.0 / 64.0 * j as f32);
                    }
                }
            }

            self.cells_buffer = vertices;
        }
    }
}

impl Drawable for Gol {
    fn draw(&mut self, timestamp: std::time::Duration) {
        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, self.gl_buffer);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (self.cells_buffer.len() * std::mem::size_of::<f32>()) as gl::types::GLsizeiptr,
                self.cells_buffer.as_ptr() as *const gl::types::GLvoid,
                gl::STATIC_DRAW,
            );
            gl::DrawArrays(
                gl::POINTS,
                0,
                (self.cells_buffer.len() / 2) as i32,
            );
        }

        let second = Duration::from_secs(1);
        if self.frame_counter != self.period_frames {
            self.frame_counter += 1;
        } else {
            self.update_cells();
            self.frame_counter = 0;
        }
        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        }
    }
}

fn main() {
    let mut ctx = RenderCtx::create();
    let mut win = Window::new(&mut ctx, 800, 800).unwrap();
    win.set_bg_col((0.2, 0.2, 0.2, 1.0));

    let mut gol = Gol::new();

    let vtx_path = "shaders/gol.vert";
    let vtx = fs::read_to_string(vtx_path)
        .expect(format!("Could not read vertex src from {}", vtx_path).as_str());

    let frag_path = "shaders/gol.frag";
    let frag = fs::read_to_string(frag_path)
        .expect(format!("Could not read vertex src frag {}", frag_path).as_str());

    let prog = unsafe { RenderCtx::compile_shader_from_src(&vtx, &frag).unwrap() };

    unsafe {
        gl::UseProgram(prog);
        gl::PointSize(win.width as f32 / 64.0);

        let mut array: gl::types::GLuint = 0;
        gl::GenVertexArrays(1, &mut array);
        gl::BindVertexArray(array);

        let mut buffer: gl::types::GLuint = 0;
        gl::GenBuffers(1, &mut buffer);

        gl::BindBuffer(gl::ARRAY_BUFFER, buffer);

        gl::EnableVertexAttribArray(0);
        gl::VertexAttribPointer(
            0,
            2,
            gl::FLOAT,
            gl::FALSE,
            (2 * std::mem::size_of::<f32>()) as gl::types::GLint,
            std::ptr::null(),
        );

        gol.gl_array = array;
        gol.gl_buffer = buffer;
    }
    
    gol.cells[32][30] = 1;
    gol.cells[30][31] = 1;
    gol.cells[32][31] = 1;
    gol.cells[31][32] = 1;
    gol.cells[32][32] = 1;

    let mut vertices = Vec::<f32>::new();
    for i in 0..gol.cells.len() {
        for j in 0..gol.cells[0].len() {
            if gol.cells[i][j] > 0 {
                let pixel_x = win.width as f32 / 64.0 * i as f32 + win.width as f32 / (64.0 * 2.0);
                let pixel_y = win.height as f32 / 64.0 * j as f32 + win.height as f32 / (64.0 * 2.0);
                vertices.push(-1.0 + 2.0 * (pixel_x / win.width as f32));
                vertices.push(1.0 - 2.0 * (pixel_y / win.height as f32));
            }
        }
    }

    gol.cells_buffer = vertices;

    win.set_draw_object(Box::new(gol));
    win.run_event_loop(&mut ctx);
}
