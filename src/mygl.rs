use gl;
use gl::types::{GLboolean, GLchar, GLenum, GLint, GLuint, GLvoid};
use std::ffi::CStr;
use std::ptr;
use freetype;
use crate::vec2::Vec2;

const VERTEX: &'static [GLint; 8] = &[
    -1, -1,
     1, -1,
     1,  1,
    -1,  1
];

const INDEXES: &'static [GLuint; 6] = &[0, 1, 2, 0, 2, 3];

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

pub unsafe fn create_font_texture() -> u32 {
    // https://togglebit.io/posts/rust-opengl-part-3/

    // Create one new texture.
    let mut texture_id = 0;
    gl::GenTextures(1, &mut texture_id);

    // Select the new texture so that we can edit it.
    gl::BindTexture(gl::TEXTURE_2D,  texture_id);

    let (texture_size, texture) = load_font();

    // Load the font texture into opengl.
    // https://www.khronos.org/registry/OpenGL-Refpages/gl4/html/glTexImage2D.xhtml
    gl::TexImage2D(
        // We want to edit the currently selected TEXTURE_2D
        gl::TEXTURE_2D,

        // Level of detail. 0 is just the base image.
        0,
        
        // Number of color numbers in each pixels.
        gl::R8 as GLint,

        // Size of the texture in pixels.
        1, // texture_size.x as i32,
        1, // texture_size.y as i32,

        // Legacy paramater, must be 0.
        0,

        // What do the color component represent?
        gl::ALPHA,

        // What number type should represent each number. 
        gl::UNSIGNED_BYTE,

        // And finaly, the actuall data.
        vec![ 0u8, 0u8, 0u8, 0u8 ].as_ptr().cast() // texture.as_ptr().cast()
    );
    
    return texture_id
}

const WIDTH: usize = 32;
const HEIGHT: usize = 24;

pub fn load_font() -> (Vec2<usize>, Vec<u8>) {
    // Path to font.
    let path = "/nix/store/krgyqigzhx2jd4i9kp104b5wkkk6gn3j-dejavu-fonts-2.37/share/fonts/truetype/DejaVuSansMono.ttf";

    // Initilize the library.
    let lib = freetype::Library::init().expect("Could not load freetype");
    
    // Load the font.
    let face = lib.new_face(path, 0).expect("Could not find font");

    // Set the font size
    face.set_char_size(40 * 64, 0, 50, 0).unwrap();

    // What characters do we want to load?
    let glyphs = "abcdef";

    // Create a 2d buffer for the texture.
    let texture_size = Vec2::new(WIDTH * glyphs.len(), HEIGHT);
    let mut texture = vec![0u8; texture_size.x * texture_size.y];

    // Rasturize each character and add them to the buffer.
    for (i, chr) in glyphs.char_indices() {
        // Rasturize the character.
        face.load_char(chr as usize, freetype::face::LoadFlag::RENDER).unwrap();
        let glyph = face.glyph();

        let offset = Vec2::new(
            glyph.bitmap_left() as usize,
            HEIGHT - glyph.bitmap_top() as usize
        );

        let bitmap = glyph.bitmap();

        let size = Vec2::new(bitmap.width() as usize, bitmap.rows() as usize);

        let buffer = bitmap.buffer();

        for x in offset.x..offset.x+size.x {
            for y in offset.y..offset.y+size.x {
                let pixel = buffer[(x - offset.x) + (y - offset.y) * size.x];
                texture[ x + i * WIDTH + y * texture_size.x ] = pixel;
            }
        }
    }

    return (texture_size, texture);
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

