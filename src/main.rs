mod mygl;
mod vec2;
mod window;

use khronos_egl::{
    Config, Context, Display, NativeWindowType, API as egl, BLUE_SIZE, CONTEXT_MAJOR_VERSION,
    CONTEXT_MINOR_VERSION, CONTEXT_OPENGL_CORE_PROFILE_BIT, CONTEXT_OPENGL_PROFILE_MASK,
    GREEN_SIZE, NONE, OPENGL_API, RED_SIZE,
};
use mygl::render;
use std::process::exit;
use vec2::Vec2;
use wayland_client::protocol::{wl_keyboard, wl_pointer, wl_seat, wl_surface};
use wayland_client::{event_enum, Filter, Main};
use wayland_protocols::xdg_shell::client::{xdg_surface, xdg_toplevel};
use window::{setup_wayland, DisplayConnection};

event_enum!(
    Event |
    Mouse => wl_pointer::WlPointer,
    Keyboard => wl_keyboard::WlKeyboard
);

fn create_context(display: Display) -> (Context, Config) {
    let attributes = [RED_SIZE, 8, GREEN_SIZE, 8, BLUE_SIZE, 8, NONE];

    let config = egl
        .choose_first_config(display, &attributes)
        .expect("unable to choose an EGL configuration")
        .expect("no EGL configuration found");

    let context_attributes = [
        CONTEXT_MAJOR_VERSION,
        4,
        CONTEXT_MINOR_VERSION,
        0,
        CONTEXT_OPENGL_PROFILE_MASK,
        CONTEXT_OPENGL_CORE_PROFILE_BIT,
        NONE,
    ];

    let context = egl
        .create_context(display, config, None, &context_attributes)
        .expect("unable to create an EGL context");

    (context, config)
}

fn create_surface(
    connection: &mut DisplayConnection,
    name: String,
    size: Vec2<i32>,
) -> (Main<wl_surface::WlSurface>, NativeWindowType) {
    // Wl surfaces represent an output that is visible to the user.
    // https://wayland-book.com/surfaces-in-depth.html
    let surface = connection.compositor.create_surface();

    // Wraper around WlSurface to add the EGL capabilities.
    // https://docs.rs/wayland-egl/0.28.5/wayland_egl/struct.WlEglSurface.html
    let egl_surface = wayland_egl::WlEglSurface::new(&surface, size.x, size.y);
    let egl_surface_ptr = egl_surface.ptr() as NativeWindowType;

    // Xdg surfaces are any wl surface that are managed by the xdg extension.
    // https://wayland-book.com/xdg-shell-basics/xdg-surface.html
    let xdg_surface = connection.xdg.get_xdg_surface(&surface);
    xdg_surface.quick_assign(move |xdg_surface, event, _| match event {
        xdg_surface::Event::Configure { serial } => {
            xdg_surface.ack_configure(serial);
        }
        _ => (),
    });

    // In xdg toplevel windows represent the main window of an app. ie: Not popup windows, ect..
    // https://wayland-book.com/xdg-shell-basics/xdg-toplevel.html
    let xdg_toplevel = xdg_surface.get_toplevel();
    xdg_toplevel.set_title(name);
    xdg_toplevel.quick_assign(move |_, event, _| match event {
        xdg_toplevel::Event::Close => exit(0),
        xdg_toplevel::Event::Configure {
            width,
            height,
            states: _,
        } => {
            println!("redraw! size: ({}, {})", width, height);

            // Resize the surface.
            egl_surface.resize(width, height, 0, 0);
        }
        _ => {
            unreachable!()
        }
    });

    // write out what we currently have and sync it to make sure the surface is configured.
    surface.commit();
    connection.sync();
    println!("done syncing.");

    // return the surface
    return (surface, egl_surface_ptr);
}

fn main() {
    // Setup OpenGL and EGL. This binds all the functions pointers to the correct functions.
    egl.bind_api(OPENGL_API)
        .expect("unable to select OpenGL API");
    gl::load_with(|name| egl.get_proc_address(name).unwrap() as *const std::ffi::c_void);

    // Setup the Wayland client.
    let mut connection = setup_wayland();

    // Create a surface.
    // Note that it must be kept alive to the end of execution.
    // https://wayland-book.com/surfaces-in-depth.html
    let (_surface, wl_egl_surface_ptr) = create_surface(
        &mut connection,
        // The title of the window.
        "term".to_string(),
        // The inital size of the window. Does not matter for sway.
        Vec2::new(100, 100),
    );

    // Setup EGL.
    let display_ptr = connection.display.get_display_ptr() as *mut std::ffi::c_void;
    let egl_display = egl.get_display(display_ptr).unwrap();
    egl.initialize(egl_display).unwrap();
    let (egl_context, egl_config) = create_context(egl_display);

    // Creates the EGL representation of the window.
    // https://www.khronos.org/registry/EGL/sdk/docs/man/html/eglCreateWindowSurface.xhtml
    let egl_surface = unsafe {
        egl.create_window_surface(egl_display, egl_config, wl_egl_surface_ptr, None)
            .expect("unable to create an EGL surface")
    };

    // Binds OpenGL context to the current thread and to the selected surface.
    // https://www.khronos.org/registry/EGL/sdk/docs/man/html/eglMakeCurrent.xhtml
    egl.make_current(
        egl_display,
        Some(egl_surface),
        Some(egl_surface),
        Some(egl_context),
    )
    .expect("unable to bind the context");

    // Render to inactive buffer, the switch with the active one.
    // https://www.khronos.org/registry/EGL/sdk/docs/man/html/eglSwapBuffers.xhtml
    render();
    egl.swap_buffers(egl_display, egl_surface).unwrap();

    /* let event_filter = Filter::new(move |event, _, _| match event {
    }); */

    // Run the main event loop.
    // https://docs.rs/wayland-client/0.28.5/wayland_client/struct.EventQueue.html
    loop {
        connection
            .event_queue
            .dispatch(&mut (), |_, _, _| {
                println!("event");
            })
            .unwrap();
    }
}
