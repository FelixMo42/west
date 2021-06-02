mod mygl;
mod vec2;
mod window;

use khronos_egl::{
    Config, Context, Display, NativeWindowType, API as egl, BLUE_SIZE, CONTEXT_MAJOR_VERSION,
    CONTEXT_MINOR_VERSION, CONTEXT_OPENGL_CORE_PROFILE_BIT, CONTEXT_OPENGL_PROFILE_MASK,
    GREEN_SIZE, NONE, OPENGL_API, RED_SIZE,
};
use mygl::{compile_program, create_font_texture, render};
use std::process::exit;
use vec2::Vec2;
use wayland_client::protocol::{wl_keyboard, wl_pointer, wl_seat};
use wayland_client::{Filter, event_enum};
use wayland_protocols::xdg_shell::client::{xdg_surface, xdg_toplevel};
use window::setup_wayland;

pub enum WindowEvent {
    Resize(Vec2<i32>)
}

event_enum!(
    Event |
    Mouse => wl_pointer::WlPointer,
    Keyboard => wl_keyboard::WlKeyboard |
    Window => WindowEvent
);

fn create_context(display: Display) -> (Context, Config) {
    let attributes = [RED_SIZE, 8, GREEN_SIZE, 8, BLUE_SIZE, 8, NONE];

    let config = egl
        .choose_first_config(display, &attributes)
        .expect("unable to choose an EGL configuration")
        .expect("no EGL configuration found");

    let context_attributes = [
        CONTEXT_MAJOR_VERSION, 4,
        CONTEXT_MINOR_VERSION, 4,
        CONTEXT_OPENGL_PROFILE_MASK, CONTEXT_OPENGL_CORE_PROFILE_BIT,
        NONE,
    ];

    let context = egl
        .create_context(display, config, None, &context_attributes)
        .expect("unable to create an EGL context");

    (context, config)
}

fn main() {
    // Setup OpenGL and EGL. This binds all the functions pointers to the correct functions.
    egl.bind_api(OPENGL_API)
        .expect("unable to select OpenGL API");
    gl::load_with(|name| egl.get_proc_address(name).unwrap() as *const std::ffi::c_void);

    // Setup the Wayland client.
    let mut connection = setup_wayland();

    // Setup EGL.
    let display_ptr = connection.display.get_display_ptr() as *mut std::ffi::c_void;
    let egl_display = egl.get_display(display_ptr).unwrap();
    egl.initialize(egl_display).unwrap();
    let (egl_context, egl_config) = create_context(egl_display);

    // Wl surfaces represent an output that is visible to the user.
    // https://wayland-book.com/surfaces-in-depth.html
    let surface = connection.compositor.create_surface();

    // Wraper around WlSurface to add the EGL capabilities.
    // https://docs.rs/wayland-egl/0.28.5/wayland_egl/struct.WlEglSurface.html
    let wl_egl_surface = wayland_egl::WlEglSurface::new(&surface, 100, 100);
    let wl_egl_surface_ptr = wl_egl_surface.ptr() as NativeWindowType;

    // Creates the EGL representation of the window.
    // https://www.khronos.org/registry/EGL/sdk/docs/man/html/eglCreateWindowSurface.xhtml
    let egl_surface = unsafe {
        egl.create_window_surface(egl_display, egl_config, wl_egl_surface_ptr, None)
            .expect("unable to create an EGL surface")
    };

    // Callback for varius types of events.
    let event_filter = Filter::new(move |event, _, _| match event {
        Event::Keyboard { event, .. } => match event {
            wl_keyboard::Event::Enter { .. } => {
            }
            wl_keyboard::Event::Leave { .. } => {
            }
            wl_keyboard::Event::Key { key: _, state: _, .. } => {
            }
            _ => {}
        }
        Event::Mouse { event, .. } => match event {
            wl_pointer::Event::Motion { surface_x: _, surface_y: _, .. } => {
            },
            wl_pointer::Event::Button { button: _, state: _, .. } => {
                println!("mouse clicked");
            }
            _ => {}
        }
        Event::Window(event) => match event {
            WindowEvent::Resize(_size) => {
                render();
                egl.swap_buffers(egl_display, egl_surface).unwrap();
            }
        } 
    });


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
    xdg_toplevel.set_title("term".to_string());
    let event_filter2 = event_filter.clone();
    xdg_toplevel.quick_assign(move |_, event, dispatch_data| match event {
        xdg_toplevel::Event::Close => exit(0),
        xdg_toplevel::Event::Configure { width, height, states: _ } => {
            let (prev_width, prev_height) = wl_egl_surface.get_size();
            
            if (width != 0 && height != 0) && (width != prev_width || height != prev_height) {
                // Resize the egl surface.
                wl_egl_surface.resize(width, height, 0, 0);

                // Tell OpenGL the new size of the window.
                // https://docs.microsoft.com/en-us/windows/win32/opengl/glviewport
                unsafe { gl::Viewport(0, 0, width, height); }

                // Send the event to the event manageger.
                let size = Vec2::new(width, height);
                let event = Event::Window(WindowEvent::Resize(size));
                event_filter2.send(event, dispatch_data);
            }
        }
        _ => {
            unreachable!()
        }
    });

    // write out what we currently have and sync it to make sure the surface is configured.
    surface.commit();
    connection.sync();

    // Binds OpenGL context to the current thread and to the selected surface.
    // https://www.khronos.org/registry/EGL/sdk/docs/man/html/eglMakeCurrent.xhtml
    egl.make_current(
        egl_display,
        Some(egl_surface),
        Some(egl_surface),
        Some(egl_context),
    )
    .expect("unable to bind the context");

    // Set up OpenGL
    unsafe {
        compile_program(); 
        create_font_texture();
    }

    // Render to inactive buffer, the switch with the active one.
    // This will get draw over beffor ever being seen, but we must draw something for
    // sway to start displaying the window.
    // https://www.khronos.org/registry/EGL/sdk/docs/man/html/eglSwapBuffers.xhtml
    render();
    egl.swap_buffers(egl_display, egl_surface).unwrap();

    // A wayland seat represents a users input divices.
    // https://wayland-book.com/seat.html
    connection
        .globals
        .instantiate_exact::<wl_seat::WlSeat>(1)
        .unwrap()
        .quick_assign(move |seat, event, _| {
            if let wl_seat::Event::Capabilities { capabilities } = event {
                println!("seat inilized.");

                if capabilities.contains(wl_seat::Capability::Pointer) {
                    println!("mouse inilized.");
                    seat.get_pointer().assign(event_filter.clone());
                }

                if capabilities.contains(wl_seat::Capability::Keyboard) {
                    println!("keyboard inilized.");
                    seat.get_keyboard().assign(event_filter.clone());
                }
            }
        });
    
    // Run the main event loop.
    // https://docs.rs/wayland-client/0.28.5/wayland_client/struct.EventQueue.html
    loop {
        connection
            .event_queue
            .dispatch(&mut (), |_, _, _| {
                println!("unhandled event");
            })
            .unwrap();
    }
}
