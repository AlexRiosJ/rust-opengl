use gl;
use sdl2;

#[macro_use]
extern crate failure;
#[macro_use]
extern crate render_gl_derive;

pub mod render_gl;
pub mod resources;

use crate::resources::Resources;
use failure::err_msg;
use gl::types::*;
use render_gl::data;
use std::path::Path;

#[derive(VertexAttribPointers, Copy, Clone, Debug)]
#[repr(C, packed)]
struct Vertex {
    #[location = "0"]
    pos: data::Vector3,
    #[location = "1"]
    clr: data::Vector3,
}

fn main() {
    if let Err(e) = run() {
        println!("{}", failure_to_string(e));
    }
}

fn run() -> Result<(), failure::Error> {
    let res = Resources::from_relative_exe_path(Path::new("assets")).unwrap();

    let sdl = sdl2::init().map_err(err_msg)?;
    let video_subsystem = sdl.video().map_err(err_msg)?;

    let gl_attr = video_subsystem.gl_attr();

    gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
    gl_attr.set_context_version(4, 5);

    let window = video_subsystem
        .window("Game", 1280, 720)
        .opengl()
        .resizable()
        .build()
        .unwrap();

    let _gl_context = window.gl_create_context().map_err(err_msg)?;
    let gl = gl::Gl::load_with(|s| {
        video_subsystem.gl_get_proc_address(s) as *const std::os::raw::c_void
    });

    let shader_program = render_gl::Program::from_res(&gl, &res, "shaders/triangle")?;

    let vertices: Vec<Vertex> = vec![
        Vertex {
            pos: (0.5, -0.5, 0.0).into(),
            clr: (1.0, 0.0, 0.0).into(),
        }, // bottom right
        Vertex {
            pos: (-0.5, -0.5, 0.0).into(),
            clr: (0.0, 1.0, 0.0).into(),
        }, // bottom left
        Vertex {
            pos: (0.0, 0.5, 0.0).into(),
            clr: (0.0, 0.0, 1.0).into(),
        }, // top
    ];

    let mut vbo: GLuint = 0;
    unsafe {
        gl.GenBuffers(1, &mut vbo);
    }

    unsafe {
        gl.BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl.BufferData(
            gl::ARRAY_BUFFER,
            (vertices.len() * std::mem::size_of::<Vertex>()) as gl::types::GLsizeiptr,
            vertices.as_ptr() as *const gl::types::GLvoid,
            gl::STATIC_DRAW,
        );
        gl.BindBuffer(gl::ARRAY_BUFFER, 0); // unbind the buffer
    }

    let mut vao: GLuint = 0;
    unsafe {
        gl.GenVertexArrays(1, &mut vao);
    }

    unsafe {
        gl.BindVertexArray(vao);
        gl.BindBuffer(gl::ARRAY_BUFFER, vbo);

        Vertex::vertex_attrib_pointers(&gl);

        gl.BindBuffer(gl::ARRAY_BUFFER, 0);
        gl.BindVertexArray(0);
    }

    unsafe {
        gl.Viewport(0, 0, 1280, 720); // set viewport
        gl.ClearColor(0.3, 0.3, 0.5, 1.0);
    }

    let mut event_pump = sdl.event_pump().unwrap();
    'main: loop {
        for event in event_pump.poll_iter() {
            match event {
                sdl2::event::Event::Quit { .. } => break 'main,
                _ => {}
            }
        }

        unsafe {
            gl.Clear(gl::COLOR_BUFFER_BIT);
        }

        shader_program.set_used();
        unsafe {
            gl.BindVertexArray(vao);
            gl.DrawArrays(
                gl::TRIANGLES, // mode
                0,             // starting index in the enabled arrays
                3,             // number of indices to be rendered
            );
        }

        window.gl_swap_window();
    }

    Ok(())
}

pub fn failure_to_string(e: failure::Error) -> String {
    use std::fmt::Write;

    let mut result = String::new();

    for (i, cause) in e
        .iter_chain()
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .enumerate()
    {
        if i > 0 {
            let _ = writeln!(&mut result, "   Which caused the following issue:");
        }
        let _ = write!(&mut result, "{}", cause);
        if let Some(backtrace) = cause.backtrace() {
            let backtrace_str = format!("{}", backtrace);
            if backtrace_str.len() > 0 {
                let _ = writeln!(&mut result, " This happened at {}", backtrace);
            } else {
                let _ = writeln!(&mut result);
            }
        } else {
            let _ = writeln!(&mut result);
        }
    }

    result
}