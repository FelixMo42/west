use gl;
use gl::types::{GLboolean, GLchar, GLenum, GLint, GLuint, GLvoid};
use std::ffi::CStr;
use std::ptr;
use freetype;
use crate::vec2::Vec2;

const VERTEX: &'static [GLint; 8] = &[-1, -1, 1, -1, 1, 1, -1, 1];

const INDEXES: &'static [GLuint; 3] = &[0, 1, 2];

const VERTEX_SHADER: &[u8] = b"#version 400
in vec2 position;
void main() {
	gl_Position = vec4(position, 0.0f, 1.0f);
}
\0";

const FRAGMENT_SHADER: &[u8] = b"#version 400
out vec4 color;
void main() {
	color = vec4(0.0f, 0.25f, 0.0f, 1.0f);
}
\0";

pub fn compile_program() -> u32 {
    load_font();

    unsafe {
        let vertex_shader = gl::CreateShader(gl::VERTEX_SHADER);
        let src = CStr::from_bytes_with_nul_unchecked(VERTEX_SHADER).as_ptr();
        gl::ShaderSource(vertex_shader, 1, (&[src]).as_ptr(), ptr::null());
        gl::CompileShader(vertex_shader);
        check_shader_status(vertex_shader);

        let fragment_shader = gl::CreateShader(gl::FRAGMENT_SHADER);
        let src = CStr::from_bytes_with_nul_unchecked(FRAGMENT_SHADER).as_ptr();
        gl::ShaderSource(fragment_shader, 1, (&[src]).as_ptr(), ptr::null());
        gl::CompileShader(fragment_shader);
        check_shader_status(fragment_shader);

        let program = gl::CreateProgram();
        gl::AttachShader(program, vertex_shader);
        gl::AttachShader(program, fragment_shader);
        gl::LinkProgram(program);
        gl::UseProgram(program);

        check_gl_errors();
        
        return program;
    }

}

const WIDTH: usize = 32;
const HEIGHT: usize = 24;

pub fn load_font() {
    // Path to font.
    let path = "/nix/store/krgyqigzhx2jd4i9kp104b5wkkk6gn3j-dejavu-fonts-2.37/share/fonts/truetype/DejaVuSansMono.ttf";

    // Initilize the library.
    let lib = freetype::Library::init().expect("Could not load freetype");
    
    // Load the font.
    let face = lib.new_face(path, 0).expect("Could not find font");

    // Set the font size
    face.set_char_size(40 * 64, 0, 50, 0).unwrap();

    for chr in "abcdef".chars() {
        face.load_char(chr as usize, freetype::face::LoadFlag::RENDER); 

        let glyph = face.glyph();

        let offset = Vec2::new(
            glyph.bitmap_left() as usize,
            HEIGHT - glyph.bitmap_top() as usize
        );

        let bitmap = glyph.bitmap();

        let size = Vec2::new(bitmap.width(), bitmap.rows());

        for y in 0..size.y {
            for x in 0..size.x {
                print!("{}", 
                    if bitmap.buffer()[(x + y * size.x) as usize] < 128 { " " } else { "#" }
                );
            }
            println!("");
        }
    }
}

pub fn render() {
    println!("rendering :o");

    unsafe {
        let num_indexes = INDEXES.len() as isize;

        let mut buffer = 0;
        gl::GenBuffers(1, &mut buffer);
        gl::BindBuffer(gl::ARRAY_BUFFER, buffer);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            8 * num_indexes,
            VERTEX.as_ptr() as *const std::ffi::c_void,
            gl::STATIC_DRAW,
        );
        check_gl_errors();

        let mut vertex_input = 0;
        gl::GenVertexArrays(1, &mut vertex_input);
        gl::BindVertexArray(vertex_input);
        gl::EnableVertexAttribArray(0);
        gl::VertexAttribPointer(0, 2, gl::INT, gl::FALSE as GLboolean, 0, 0 as *const GLvoid);
        check_gl_errors();

        let mut indexes = 0;
        gl::GenBuffers(1, &mut indexes);
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, indexes);
        gl::BufferData(
            gl::ELEMENT_ARRAY_BUFFER,
            4 * num_indexes,
            INDEXES.as_ptr() as *const std::ffi::c_void,
            gl::STATIC_DRAW,
        );
        check_gl_errors();

        gl::DrawElements(
            gl::TRIANGLES,
            num_indexes as i32,
            gl::UNSIGNED_INT,
            std::ptr::null()
        );
        check_gl_errors();
    }
}

fn format_error(e: GLenum) -> &'static str {
    match e {
        gl::NO_ERROR => "No error",
        gl::INVALID_ENUM => "Invalid enum",
        gl::INVALID_VALUE => "Invalid value",
        gl::INVALID_OPERATION => "Invalid operation",
        gl::INVALID_FRAMEBUFFER_OPERATION => "Invalid framebuffer operation",
        gl::OUT_OF_MEMORY => "Out of memory",
        gl::STACK_UNDERFLOW => "Stack underflow",
        gl::STACK_OVERFLOW => "Stack overflow",
        _ => "Unknown error",
    }
}

pub fn check_gl_errors() {
    unsafe {
        match gl::GetError() {
            gl::NO_ERROR => (),
            e => {
                panic!("OpenGL error: {}", format_error(e))
            }
        }
    }
}

unsafe fn check_shader_status(shader: GLuint) {
    let mut status = gl::FALSE as GLint;
    gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut status);
    if status != (gl::TRUE as GLint) {
        let mut len = 0;
        gl::GetProgramiv(shader, gl::INFO_LOG_LENGTH, &mut len);
        if len > 0 {
            let mut buf = Vec::with_capacity(len as usize);
            buf.set_len((len as usize) - 1); // subtract 1 to skip the trailing null character
            gl::GetProgramInfoLog(
                shader,
                len,
                ptr::null_mut(),
                buf.as_mut_ptr() as *mut GLchar,
            );

            let log = String::from_utf8(buf).unwrap();
            eprintln!("shader compilation log:\n{}", log);
        }

        panic!("shader compilation failed");
    }
}

