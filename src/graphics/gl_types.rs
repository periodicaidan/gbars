//! A simple set of wrappers around the OpenGL API.
//!
//! I opted to use raw OpenGL calls to handle the graphics for the emulator instead of using a
//! library because all I need to do is draw a textured rectangle to a frame buffer. I don't need or
//! want the extra overhead of a proper library. However, although this is fairly simple and could
//! be done in a single function, it still requires hundreds of lines of cryptic, unsafe function
//! calls. So this is a small library that wraps those function calls in a safe interface, providing
//! a documented level of indirection so readers of the code can at least somewhat understand what's
//! going on.
//!
//! Obviously, a knowledge of OpenGL is *necessary* to understand any of this, so let me offer some
//! of the resources I found informative. [Learn OpenGL](learnopengl) is a classic set of
//! somewhat-old tutorials on understanding and using OpenGL, using C++ examples. If you'd like to
//! follow along in Rust, there is [a Rust port of all the example code](learnopengl-rs). Finally,
//! much of this code was inspired by [Rust and OpenGL from scratch](rs-opengl-from-scratch), which
//! is a brilliant tutorial on rolling your own OpenGL library in Rust, and on how to write a safe
//! interface over unsafe code.
//!
//! [learnopengl]: https://learnopengl.com/
//! [learnopengl-rs]: https://github.com/bwasty/learn-opengl-rs
//! [rs-opengl-from-scratch]: http://nercury.github.io/rust/opengl/tutorial/2018/02/09/opengl-in-rust-from-scratch-02-opengl-context.html

use std::ffi::{CString, CStr};
use std::mem::size_of;
use std::ptr::{null, null_mut};

use gl;
use gl::types::*;

/// Represents a compiled shader.
pub struct GlShader(GLuint);

/// Represents the graphics pipeline.
pub struct GlProgram(GLuint);

/// Represents a [vertex buffer object (VBO)](vbo), which is a representation of vertex data that's
/// sent to the graphics card. Vertices don't have to be spatial; they can represent color, normal
/// vectors, or any other data you want sent to the graphics card.
///
/// [vbo]: https://en.wikipedia.org/wiki/Vertex_buffer_object
pub struct GlVertexBuffer;

/// An object that abstracts over the arguments of [gl::VertexAttribPointer](glvertexattribpointer)
/// function, which tells OpenGL how to read vertex data.
///
/// [glvertexattribpointer]: http://docs.gl/gl3/glVertexAttribPointer
pub struct GlVertexAttribute {
    index: GLuint,
    size: GLint,
    kind: GLenum,
    stride: GLsizei,
    start: usize,
}

/// Represents an element buffer object (EBO), is a list of vertex ID's. This allows you to reuse
/// vertex data. This way, to draw a rectangle (which is two triangles that share a common side),
/// you don't need to send data for the shared vertices twice.
pub struct GlElementBuffer;

pub struct GlFrameBuffer;

/// "Texture" is the OpenGL term for an image passed to the graphics pipeline. In the fragment
/// shader (which specifies the colors of the pixels, or "fragments"), a texture can be sampled and
/// its pixel data applied to the pixels in the pipeline.
pub struct GlTexture;