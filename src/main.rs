#[macro_use] extern crate clap;
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate bitmatch;

//pub mod interface;
pub mod classic;
pub mod ips;
//pub mod graphics;
//pub mod emu;
//pub mod ips;
//pub mod audio;

//use interface::cli::cli_main;
//use interface::gui::gui_main;

use std::vec::Vec;

use std::thread;

use std::path::Path;

use std::env;

use glutin::{
    event_loop::{
        EventLoop,
        ControlFlow
    },
    window::{
        WindowBuilder,
        Window
    },
    event::{
        Event,
        WindowEvent
    }
};

//use self::graphics::gl_types::*;
use std::fs::File;

const STACK_SIZE: usize = 0x4000000;

enum GameBoyConsoleType {
//    Classic(classic::gb_types::Console)
}


fn run() {
//    let mut console = classic::gb_types::Console::init(
//        Some("src/test_roms/pokeblue.gbc")
//    );
//
//    let initial_height = 144.0;
//    let initial_width = 160.0;
//
//    let events = EventLoop::new();
//    let window = WindowBuilder::new()
//        .with_title("GBARS")
//        .with_inner_size(glutin::dpi::LogicalSize::new(initial_width, initial_height));
//    let win_context = glutin::ContextBuilder::new()
//        .build_windowed(window, &events)
//        .unwrap();
//
//    let win_context = unsafe {
//        win_context.make_current().unwrap()
//    };
//
//    gl::load_with(|s| win_context.get_proc_address(s) as *const std::ffi::c_void);
//
//    let mut screen = classic::gb_types::ScreenBuffer{
//        pixels: Vec::with_capacity(320 * 320),
//        scale: 1,
//        scy: 0,
//        scx: 0,
//        ly: 0,
//        lyc: 0,
//        wy: 0,
//        wx: 0
//    };
//
//    screen.pixels.extend([3, 0].iter().cycle().take(320 * 320));
//
//    let vertices: Vec<f32> = vec![
//        // Position     Texture
//        -1.0, 1.0,      0.0, 0.0,
//        1.0,  1.0,      1.0, 0.0,
//        1.0,  -1.0,     1.0, 1.0,
//        -1.0, -1.0,     0.0, 1.0
//    ];
//
//    let elements: Vec<u32> = vec![
//        0, 1, 2,
//        2, 3, 0
//    ];
//
//    let mut vao = 0u32;
//    unsafe {
//        gl::GenVertexArrays(1, &mut vao);
//        gl::BindVertexArray(vao);
//    }
//
//    let tex = GlTexture::from_screen(&screen).unwrap();
//    let vbo = GlVertexBuffer::init(&vertices);
//    let ebo = GlElementBuffer::init(&elements);
//
//    set_vertex_attrib(0, 0, 2, 4);
//    set_vertex_attrib(1, 2, 2, 4);
//
//    unbind_buffers(GlBufferType::Array);
//
//    let vert_shader = GlShader::from_vert_source(
//        &format!("{}/src/graphics/shaders/gb_screen.vert", env::current_dir().unwrap().to_str().unwrap())
//    ).unwrap();
//
//    let frag_shader = GlShader::from_frag_source(
//        &format!("{}/src/graphics/shaders/gb_screen.frag", env::current_dir().unwrap().to_str().unwrap())
//    ).unwrap();
//
//    let program = GlProgram::from_shaders(&[vert_shader, frag_shader]).unwrap();
//
//    events.run(move |event, _, control_flow| {
//        let now = std::time::Instant::now();
//        *control_flow = ControlFlow::Wait;
//        let mut size: glutin::dpi::LogicalSize = win_context.window().inner_size();
//        let (mut width, mut height) = (size.width, size.height);
//        let (mut bottom, mut left) = (0.0, 0.0);
//
//        if width * initial_height > height * initial_width {
//            let device_width = width;
//            width = height * initial_width / initial_height;
//            left = (device_width - width) / 2.0;
//        } else {
//            let device_height = height;
//            height = width * initial_height / initial_width;
//            bottom = (device_height - height) / 2.0;
//        }
//
//        match event {
//            Event::WindowEvent { ref event, .. } => match event {
//                WindowEvent::RedrawRequested => {
//                    unsafe {
//                        gl::Viewport(left as i32, bottom as i32, width as i32, height as i32);
//                        gl::ClearColor(0.0, 0.0, 0.0, 1.0);
//                        gl::Clear(gl::COLOR_BUFFER_BIT);
//                    }
//
//                    program.set_used();
//                    unsafe {
//                        gl::BindVertexArray(vao);
//                        gl::DrawElements(
//                            gl::TRIANGLES,
//                            6,
//                            gl::UNSIGNED_INT,
//                            std::ptr::null()
//                        )
//                    }
//
//                    win_context.swap_buffers().unwrap();
//                },
//                WindowEvent::Resized(logical_size) => {
//                    let dpi_factor = win_context.window().hidpi_factor();
//                    win_context.resize(logical_size.to_physical(dpi_factor));
//                },
//                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
//                _ => {}
//            },
//
//            _ => {}
//        }
//
//        console.step();
//    })
}


fn main() {
    let child = thread::Builder::new()
        .stack_size(STACK_SIZE)
        .name(String::from("gbars"))
        .spawn(run)
        .unwrap();

    child.join().unwrap();
}
