use gl;
// NB: I don't bother with the GL types because they're all type aliases for Rust primitives
use glutin::event_loop::{EventLoop, ControlFlow};
use glutin::window::WindowBuilder;
use glutin::event::{Event, WindowEvent};
use std::vec::Vec;
use std::mem::size_of;
use std::ffi::{CString, CStr, c_void};
use std::ptr::{null, null_mut};

use crate::classic::gb_types::ScreenBuffer as ClassicScreen;

use super::utils::*;
use std::path::Path;
use std::fs::File;
use std::error::Error;
use std::io::Read;

pub enum GlShaderType {
    Vertex = gl::VERTEX_SHADER as isize,
    Fragment = gl::FRAGMENT_SHADER as isize
}

pub struct GlShader {
    id: u32
}

impl GlShader {
    pub fn id(&self) -> u32 { self.id }

    pub fn from_source(src: &str, kind: GlShaderType) -> Result<Self, String> {
        let file = File::open(src);

        if let Err(e) = file {
            return Err(format!("Error opening file {}: {}", src, e.description()));
        }
        let mut contents: Vec<u8> = vec![];
        file.unwrap()
            .read_to_end(&mut contents).unwrap();

        let id = shader_from_source(
            unsafe{ &CString::from_vec_unchecked(contents) },
            kind as u32
        )?;

        Ok(Self { id })
    }

    pub fn from_vert_source(src: &str) -> Result<Self, String> {
        Self::from_source(src, GlShaderType::Vertex)
    }

    pub fn from_frag_source(src: &str) -> Result<Self, String> {
        Self::from_source(src, GlShaderType::Fragment)
    }
}

impl Drop for GlShader {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteShader(self.id);
        }
    }
}

pub struct GlProgram {
    id: u32
}

impl GlProgram {
    pub fn id(&self) -> u32 { self.id }

    pub fn from_shaders(shaders: &[GlShader]) -> Result<Self, String> {
        let id = unsafe { gl::CreateProgram() };

        unsafe {
            for shader in shaders {
                gl::AttachShader(id, shader.id());
            }

            gl::LinkProgram(id);

            let mut success = 1;
            unsafe {
                gl::GetProgramiv(id, gl::LINK_STATUS, &mut success);
            }

            if success == 0 {
                let mut len = 0;
                unsafe {
                    gl::GetProgramiv(id, gl::INFO_LOG_LENGTH, &mut len);
                }

                let mut error = create_ws_cstring_with_len(len as usize);
                gl::GetProgramInfoLog(id, len, null_mut(), error.as_ptr() as *mut i8);


                return Err(error.to_string_lossy().into_owned());
            }

            for shader in shaders {
                gl::DetachShader(id, shader.id());
            }
        }

        Ok(Self { id })
    }

    pub fn set_used(&self) {
        unsafe {
            gl::UseProgram(self.id);
        }
    }
}

impl Drop for GlProgram {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.id);
        }
    }
}

pub struct GlTexture {
    id: u32,
}

impl GlTexture {
    pub fn id(&self) -> u32 { self.id }

    pub fn from_screen(screen: &ClassicScreen) -> Result<Self, String> {
        let mut id: u32 = 0;
        unsafe {
            gl::GenTextures(1, &mut id);
            gl::BindTexture(gl::TEXTURE_2D, id);

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);

            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGB as i32,
                ClassicScreen::VISIBLE_X as i32,
                ClassicScreen::VISIBLE_Y as i32,
                0,
                gl::RGB,
                gl::FLOAT,
                screen.gl_rgb_pixels().as_ptr() as *const c_void
            );
        }

        Ok(GlTexture { id })
    }
}

pub struct GlFrameBuffer {
    id: u32
}

impl GlFrameBuffer {
    pub fn id(&self) -> u32 { self.id }
}

impl From<ClassicScreen> for Result<GlFrameBuffer, String> {
    fn from(screen: ClassicScreen) -> Self {
        let mut id: u32 = 0;
        unsafe {
            gl::GenFramebuffers(1, &mut id);
            gl::BindFramebuffer(gl::FRAMEBUFFER, id);

            let tex = GlTexture::from_screen(&screen)?;

            gl::FramebufferTexture2D(
                gl::FRAMEBUFFER,
                gl::COLOR_ATTACHMENT0,
                gl::TEXTURE_2D,
                tex.id(),
                0
            );
        }

        Ok(GlFrameBuffer { id })
    }
}

impl Drop for GlFrameBuffer {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteFramebuffers(1, &mut self.id);
        }
    }
}

pub struct GlWindow {
    title: String,
    width: usize,
    height: usize
}

pub enum GlBufferType {
    Array = gl::ARRAY_BUFFER as isize,
    Element = gl::ELEMENT_ARRAY_BUFFER as isize
}

pub struct GlVertexBuffer {
    id: u32
}

impl GlVertexBuffer {
    pub fn id(&self) -> u32 { self.id }

    pub fn init(data: &[f32]) -> Self {
        let mut vbo = Self::generate();
        vbo.bind();
        unsafe {
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (data.len() * size_of::<f32>()) as isize,
                data.as_ptr() as *const c_void,
                gl::STATIC_DRAW
            );
        }

        vbo
    }

    pub fn generate() -> Self {
        let mut vbo = 0u32;
        unsafe { gl::GenBuffers(1, &mut vbo) };
        Self {
            id: vbo
        }
    }

    pub fn bind(&self) {
        unsafe { gl::BindBuffer(gl::ARRAY_BUFFER, self.id); }
    }
}

impl Drop for GlVertexBuffer {
    fn drop(&mut self) {
        unbind_buffers(GlBufferType::Array);
    }
}

pub struct GlElementBuffer {
    id: u32
}

impl GlElementBuffer {
    pub fn id(&self) -> u32 { self.id }

    pub fn init(data: &[u32]) -> Self {
        let mut ebo = Self::generate();
        ebo.bind();
        unsafe {
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (data.len() * size_of::<u32>()) as isize,
                data.as_ptr() as *const c_void,
                gl::STATIC_DRAW
            );
        }

        ebo
    }

    pub fn generate() -> Self {
        let mut ebo = 0u32;
        unsafe { gl::GenBuffers(1, &mut ebo) };
        Self {
            id: ebo
        }
    }

    pub fn bind(&self) {
        unsafe { gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.id); }
    }
}

impl Drop for GlElementBuffer {
    fn drop(&mut self) {
        unbind_buffers(GlBufferType::Element);
    }
}

pub fn unbind_buffers(buffer_type: GlBufferType) {
    unsafe { gl::BindBuffer(buffer_type as u32, 0); }
}

pub fn set_vertex_attrib(index: u32, offset: usize, size: i32, stride: usize) {
    unsafe {
        gl::EnableVertexAttribArray(index);
        gl::VertexAttribPointer(
            index,
            size,
            gl::FLOAT,
            gl::FALSE,
            (stride * size_of::<f32>()) as i32,
            (offset * size_of::<u32>()) as *const c_void
        );
    }
}