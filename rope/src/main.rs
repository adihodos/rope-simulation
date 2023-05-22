use std::{
    f32::consts,
    mem::size_of,
    ptr::{copy_nonoverlapping, null},
    time::{Duration, Instant},
};

use glfw::{Action, Context, Key, MouseButton, OpenGlProfileHint, WindowEvent, WindowHint};
use rand::prelude::*;

use math::{
    color_palette::{basic, material_design, turbo_srgb},
    colors::RGBAColorF32,
    projection,
    vec2::{normalize, perp_vec, Vec2F32},
    vertex_types::VertexPC,
};

use rendering::{
    create_shader_program_from_string, gl, ShaderType, UniqueBuffer, UniqueBufferMapping,
    UniquePipeline, UniqueShaderProgram, UniqueVertexArray,
};

struct DrawContext {
    vertices: Vec<VertexPC>,
    vertex_buffer: UniqueBuffer,
    vao: UniqueVertexArray,
    vertex_shader: UniqueShaderProgram,
    fragment_shader: UniqueShaderProgram,
    pipeline: UniquePipeline,
}

impl DrawContext {
    fn new(capacity: u32) -> Option<DrawContext> {
        let vertex_buffer = UniqueBuffer::new(unsafe {
            let mut handle = 0u32;
            gl::CreateBuffers(1, &mut handle as *mut _);
            gl::NamedBufferStorage(
                handle,
                (capacity as usize * size_of::<VertexPC>()) as isize,
                null(),
                gl::MAP_WRITE_BIT,
            );

            handle
        })?;

        let vao = UniqueVertexArray::new(unsafe {
            let mut handle = 0u32;
            gl::CreateVertexArrays(1, &mut handle as *mut _);
            gl::VertexArrayVertexBuffer(handle, 0, *vertex_buffer, 0, size_of::<VertexPC>() as i32);
            gl::VertexArrayAttribFormat(handle, 0, 2, gl::FLOAT, gl::FALSE, 0);
            gl::VertexArrayAttribFormat(handle, 1, 4, gl::FLOAT, gl::FALSE, 8);
            gl::VertexArrayAttribBinding(handle, 0, 0);
            gl::VertexArrayAttribBinding(handle, 1, 0);
            gl::EnableVertexArrayAttrib(handle, 0);
            gl::EnableVertexArrayAttrib(handle, 1);

            handle
        })?;

        let vertex_shader = create_shader_program_from_string(
            include_str!("../../data/shaders/ui.vert.glsl"),
            ShaderType::Vertex,
        )
        .ok()?;

        let fragment_shader = create_shader_program_from_string(
            include_str!("../../data/shaders/ui.frag.glsl"),
            ShaderType::Fragment,
        )
        .ok()?;

        let pipeline = UniquePipeline::new(unsafe {
            let mut handle = 0u32;
            gl::CreateProgramPipelines(1, &mut handle);
            gl::UseProgramStages(handle, gl::VERTEX_SHADER_BIT, *vertex_shader);
            gl::UseProgramStages(handle, gl::FRAGMENT_SHADER_BIT, *fragment_shader);

            handle
        })?;

        Some(DrawContext {
            vertices: Vec::with_capacity(capacity as usize),
            vertex_buffer,
            vao,
            vertex_shader,
            fragment_shader,
            pipeline,
        })
    }

    fn rectangle(&mut self, x: f32, y: f32, width: f32, height: f32, color: (u8, u8, u8)) {
        let vertices: [Vec2F32; 4] = [
            Vec2F32::new(x, y),
            Vec2F32::new(x + width, y),
            Vec2F32::new(x + width, y + height),
            Vec2F32::new(x, y + height),
        ];

        self.vertices
            .extend([3, 2, 0, 2, 1, 0].iter().map(|&idx| VertexPC {
                pos: vertices[idx as usize],
                color: color.into(),
            }));
    }

    fn circle(&mut self, cx: f32, cy: f32, radius: f32, color: impl Into<RGBAColorF32> + Copy) {
        const SLICES: u32 = 32;
        let angle = 2f32 * consts::PI / SLICES as f32;

        (0..SLICES).for_each(|i| {
            self.vertices.push(VertexPC {
                pos: Vec2F32::new(cx, cy),
                color: color.into(),
            });
            self.vertices.push(VertexPC {
                pos: Vec2F32::new(
                    cx + radius * ((i as f32) * angle).cos(),
                    cy + radius * ((i as f32) * angle).sin(),
                ),
                color: color.into(),
            });
            self.vertices.push(VertexPC {
                pos: Vec2F32::new(
                    cx + radius * ((i + 1) as f32 * angle).cos(),
                    cy + radius * ((i + 1) as f32 * angle).sin(),
                ),
                color: color.into(),
            });
        });
    }

    fn triangle_shaded<C: Into<RGBAColorF32> + Copy>(
        &mut self,
        x: Vec2F32,
        y: Vec2F32,
        z: Vec2F32,
        cx: C,
        cy: C,
        cz: C,
    ) {
        self.vertices.push(VertexPC {
            pos: x,
            color: cx.into(),
        });
        self.vertices.push(VertexPC {
            pos: y,
            color: cy.into(),
        });
        self.vertices.push(VertexPC {
            pos: z,
            color: cz.into(),
        });
    }

    fn line(
        &mut self,
        start: Vec2F32,
        end: Vec2F32,
        width: f32,
        color: impl Into<RGBAColorF32> + Copy,
    ) {
        assert!(width > 1f32);

        let d = end - start;
        let dp = normalize(perp_vec(d));

        let vertices = [
            start - dp * 0.5f32 * width,
            end - dp * 0.5f32 * width,
            end + dp * 0.5f32 * width,
            start + dp * 0.5f32 * width,
        ];

        self.vertices
            .extend([0, 1, 2, 0, 2, 3].iter().map(|&i| VertexPC {
                pos: vertices[i as usize],
                color: color.into(),
            }));
    }

    fn flush(&mut self, fb_width: f32, fb_height: f32) {
        if self.vertices.is_empty() {
            return;
        }

        UniqueBufferMapping::new(*self.vertex_buffer, gl::MAP_WRITE_BIT).map(|mut buf| unsafe {
            copy_nonoverlapping(
                self.vertices.as_ptr(),
                buf.as_mut_ptr(),
                self.vertices.len(),
            );
        });

        let proj_mtx = projection::orthographic(0f32, 0f32, fb_width, fb_height, -1f32, 1f32);

        unsafe {
            gl::BindVertexArray(*self.vao);
            gl::BindProgramPipeline(*self.pipeline);
            gl::ProgramUniformMatrix4fv(*self.vertex_shader, 0, 1, gl::TRUE, proj_mtx.as_ptr());
            gl::DrawArrays(gl::TRIANGLES, 0, self.vertices.len() as i32);
        }

        self.vertices.clear();
    }
}

fn main() {
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
    glfw.window_hint(WindowHint::ContextVersion(4, 5));
    glfw.window_hint(WindowHint::OpenGlForwardCompat(true));
    glfw.window_hint(WindowHint::OpenGlProfile(OpenGlProfileHint::Core));
    glfw.window_hint(WindowHint::DoubleBuffer(true));
    glfw.window_hint(WindowHint::Decorated(false));

    let (width, height) = glfw
        .with_connected_monitors(|_, mocnitors| {
            mocnitors.iter().find_map(|m| {
                let pos = m.get_pos();
                if pos.0 == 0 && pos.1 == 0 {
                    let (_, _, w, h) = m.get_workarea();
                    Some((w, h))
                } else {
                    None
                }
            })
        })
        .expect("Failed to get primary monitor size!");

    println!("Primary monitor {} -> {}", width, height);

    let (mut window, events) = glfw
        .create_window(
            width as u32,
            height as u32,
            "Rope simulation",
            glfw::WindowMode::Windowed,
        )
        .expect("Failed to create GLFW window.");

    window.set_key_polling(true);
    window.set_mouse_button_polling(true);
    window.set_cursor_pos_polling(true);
    window.make_current();

    gl::load_with(|s| window.get_proc_address(s) as *const _);

    let mut sim = {
        let (fb_width, fb_height) = window.get_framebuffer_size();
        Rope::new(fb_width, fb_height)
    };

    // let fb = Framebuffer::new(sim.fb_width as u32, sim.fb_height as u32)
    //     .expect("Failed to create framebuffer");
    let mut dc = DrawContext::new(4096).expect("Failed to create draw context!");
    let mut last_time = Instant::now();

    while !window.should_close() {
        glfw.poll_events();
        for (_, event) in glfw::flush_messages(&events) {
            handle_window_event(&mut window, &event, &mut sim);
            sim.handle_event(&mut window, &event);
        }

        let current_time = Instant::now();
        let delta_time = (current_time - last_time).as_secs_f32();
        last_time = current_time;

        unsafe {
            gl::ClearNamedFramebufferfv(
                0,
                gl::COLOR,
                0,
                (&[0.2f32, 0.2f32, 0.2f32, 1f32]).as_ptr(),
            );
            gl::ClearNamedFramebufferfi(0, gl::DEPTH_STENCIL, 0, 1f32, 0);
        }

        sim.update(&mut window, delta_time);
        sim.render(&mut dc);

        dc.flush(sim.fb_width as f32, sim.fb_height as f32);

        window.swap_buffers();
        std::thread::sleep(Duration::from_millis(10));
    }
}

struct Rope {
    fb_width: i32,
    fb_height: i32,
    knot: Vec<Vec2F32>,
    knot_velocity: Vec<Vec2F32>,
    dragged: Option<usize>,
}

impl Rope {
    const KNOT_RADIUS: f32 = 32f32;
    // const KNOT_COLOR: (u8, u8, u8) = basic::RED;
    const TARGET_DISTANCE: f32 = 200f32;
    const EPSILON: f32 = 1.0E-6f32;
    const STIFFNESS: f32 = 20f32;
    const ROPE_LENGTH: usize = 8;

    fn new(fb_width: i32, fb_height: i32) -> Rope {
        let mut rng = thread_rng();
        Rope {
            fb_width,
            fb_height,
            knot: (0..Self::ROPE_LENGTH)
                .map(|_| {
                    Vec2F32::new(
                        rng.gen_range(0f32..=1f32) * fb_width as f32,
                        rng.gen_range(0f32..=1f32) * fb_height as f32,
                    )
                })
                .collect(),
            knot_velocity: vec![Vec2F32::same(0f32); Self::ROPE_LENGTH],
            dragged: None,
        }
    }

    fn render(&mut self, draw_context: &mut DrawContext) {
        for i in 1..self.knot.len() {
            draw_context.line(self.knot[i - 1], self.knot[i], 30f32, basic::DK_GREY);
        }

        self.knot.iter().for_each(|&knot| {
            draw_context.circle(knot.x, knot.y, Self::KNOT_RADIUS, basic::GREEN);
        });
    }

    fn handle_event(&mut self, window: &mut glfw::Window, event: &glfw::WindowEvent) {
        match *event {
            WindowEvent::MouseButton(MouseButton::Button1, Action::Press, _) => {
                let (mousex, mousey) = window.get_cursor_pos();

                self.dragged = self.knot.iter().position(|&knot| {
                    (knot - Vec2F32::new(mousex as f32, mousey as f32)).square_len()
                        <= Rope::KNOT_RADIUS * Rope::KNOT_RADIUS
                });

                println!("Dragged = {:?}", self.dragged);
            }
            WindowEvent::MouseButton(MouseButton::Button1, Action::Release, _) => {
                self.dragged = None;
            }
            // WindowEvent::CursorPos(x, y) => {
            //     self.dragged.map(|dragged_knot| {
            //         self.knot[dragged_knot] = Vec2F32::new(x as f32, y as f32);
            //     });
            // }
            _ => {}
        }
    }

    fn update(&mut self, window: &mut glfw::Window, delta: f32) {
        self.dragged.map(|i| {
            let (mx, my) = window.get_cursor_pos();
            self.knot[i] = Vec2F32::new(mx as f32, my as f32);
        });

        self.knot_velocity[0] = Self::compute_knot_velocity(self.knot[0], &[self.knot[1]]);

        for i in 1..self.knot.len() - 1 {
            self.knot_velocity[i] =
                Self::compute_knot_velocity(self.knot[i], &[self.knot[i - 1], self.knot[i + 1]]);
        }

        self.knot_velocity[self.knot.len() - 1] = Self::compute_knot_velocity(
            self.knot[self.knot.len() - 1],
            &[self.knot[self.knot.len() - 2]],
        );

        for i in 0..self.knot.len() {
            let is_dragged = self.dragged.map_or(false, |d| d == i);
            if !is_dragged {
                self.knot[i] += self.knot_velocity[i] * delta;
            }
        }
    }

    fn compute_knot_velocity(knot: Vec2F32, neighbours: &[Vec2F32]) -> Vec2F32 {
        neighbours
            .iter()
            .fold(Vec2F32::same(0f32), |result, &neighbour| {
                let len = (knot - neighbour).len();
                let dir = if len > Self::EPSILON {
                    (knot - neighbour) / len
                } else {
                    Vec2F32::new(1f32, 0f32)
                };

                let target = neighbour + dir * Self::TARGET_DISTANCE;
                result + (target - knot) * Self::STIFFNESS
            })
    }
}

struct Framebuffer {
    renderbuffer_color: u32,
    depth_stencil_texture: u32,
    framebuffer: u32,
}

impl Framebuffer {
    fn new(w: u32, h: u32) -> Option<Framebuffer> {
        let renderbuffer_color = unsafe {
            let mut handle = 0u32;
            gl::CreateRenderbuffers(1, &mut handle as *mut _);
            gl::NamedRenderbufferStorage(handle, gl::RGBA8, w as i32, h as i32);

            handle
        };

        let depth_stencil_texture = unsafe {
            let mut handle = 0u32;
            gl::CreateTextures(gl::TEXTURE_2D, 1, &mut handle as *mut _);
            gl::TextureStorage2D(handle, 1, gl::DEPTH24_STENCIL8, w as i32, h as i32);

            handle
        };

        let framebuffer = unsafe {
            let mut handle = 0u32;
            gl::CreateFramebuffers(1, &mut handle as *mut _);
            gl::NamedFramebufferRenderbuffer(
                handle,
                gl::COLOR_ATTACHMENT0,
                gl::RENDERBUFFER,
                renderbuffer_color,
            );
            gl::NamedFramebufferTexture(
                handle,
                gl::DEPTH_STENCIL_ATTACHMENT,
                depth_stencil_texture,
                0,
            );

            let fb_status = gl::CheckNamedFramebufferStatus(handle, gl::FRAMEBUFFER);
            println!("framebuffer status = {}", fb_status);
            assert!(fb_status == gl::FRAMEBUFFER_COMPLETE);

            gl::ClearNamedFramebufferfv(handle, gl::COLOR, 0, (&[1f32, 0f32, 0f32, 1f32]).as_ptr());
            gl::ClearNamedFramebufferfi(handle, gl::DEPTH_STENCIL, 0, 1f32, 0);

            handle
        };

        Some(Framebuffer {
            renderbuffer_color,
            depth_stencil_texture,
            framebuffer,
        })
    }
}

fn handle_window_event(window: &mut glfw::Window, event: &glfw::WindowEvent, s: &mut Rope) {
    match *event {
        glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => window.set_should_close(true),
        WindowEvent::FramebufferSize(w, h) => {
            s.fb_width = w;
            s.fb_height = h;
        }
        _ => {}
    }
}

fn get_cursor_position(window: &glfw::Window) -> Vec2F32 {
    let (cx, cy) = window.get_cursor_pos();
    Vec2F32::new(cx as f32, cy as f32)
}
