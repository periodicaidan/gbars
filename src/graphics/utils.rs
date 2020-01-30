use gl;
use std::vec::Vec;
use std::mem::size_of;
use std::ffi::{CString, CStr, c_void};
use std::ptr::{null, null_mut};

pub fn shader_from_source(source: &CStr, kind: u32) -> Result<u32, String> {
    let id: u32 = unsafe { gl::CreateShader(kind) };
    unsafe {
        gl::ShaderSource(id, 1, &source.as_ptr(), null());
        gl::CompileShader(id);
    }

    let mut success = 1;
    unsafe {
        gl::GetShaderiv(id, gl::COMPILE_STATUS, &mut success);

        if success == 0 {
            let mut len = 0;
            unsafe {
                gl::GetShaderiv(id, gl::INFO_LOG_LENGTH, &mut len);
            }

            let mut error = create_ws_cstring_with_len(len as usize);
            unsafe {
                gl::GetShaderInfoLog(id, len, null_mut(), error.as_ptr() as *mut i8);
            }

            return Err(error // CString
                .to_string_lossy() // &str or String
                .into_owned() // String
            );
        }
    }

    Ok(id)
}

pub fn create_ws_cstring_with_len(len: usize) -> CString {
    let mut buf: Vec<u8> = Vec::with_capacity(len + 1);
    buf.extend([b' '].iter().cycle().take(len)); // Adds a bunch of spaces to the buffer
    unsafe { CString::from_vec_unchecked(buf) }
}