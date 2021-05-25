// Allow single character names so clippy doesn't lint on x, y, r, g, b, which
// are reasonable variable names in this domain.
#![allow(clippy::many_single_char_names)]

mod vec2;

use egl::Upcast;
pub use vec2::Vec2;

use std::process::exit;

use wayland_client::{
    protocol::{wl_compositor, wl_seat},
    Display, GlobalManager
};
use wayland_protocols::xdg_shell::client::{xdg_surface, xdg_toplevel, xdg_wm_base};
use khronos_egl as egl;
use gl;

fn main() {
    init_wayland_client();
}



fn init_wayland_client() {
    // Setup open Gl.
    egl::API.bind_api(egl::OPENGL_API).expect("unable to find openGL Api");
    gl::load_with(|name| egl::API.get_proc_address(name).unwrap() as *const std::ffi::c_void);
    
    // Create the wl display.
    // https://wayland-book.com/wayland-display.html
    let display = Display::connect_to_env().expect("Failed to create wl display.");
    let display_ptr = display.get_display_ptr() as *mut std::ffi::c_void;

    // Create an event queue and attach it to the display.
    // Used to send and recive messages from the compositor.
    // https://docs.rs/wayland-client/0.28.5/wayland_client/struct.EventQueue.html
    let mut event_queue = display.create_event_queue();
    let display = display.attach(event_queue.token());
    
    // Bind to wayland globals.
    // https://wayland-book.com/registry/binding.html
    let globals = GlobalManager::new(&display);
    
    // Make a synchronized roundtrip to the wayland server.
    // When this returns it must be true that the server has already
    // sent us all available globals.
    event_queue.sync_roundtrip(&mut (), |_, _, _| unreachable!()).unwrap();

    // Part of api shared between windows.
    // https://wayland-book.com/xdg-shell-basics.html
    let xdg_wm_base = globals
        .instantiate_exact::<xdg_wm_base::XdgWmBase>(2)
        .expect("Compositor does not support xdg_shell");

    // The only thing its doing here is do some ping pong to see if the app is responding.
    xdg_wm_base.quick_assign(|xdg_wm_base, event, _| {
        if let xdg_wm_base::Event::Ping { serial } = event {
            xdg_wm_base.pong(serial);
        };
    });

    // Get mouse of keyboard events.
    globals.instantiate_exact::<wl_seat::WlSeat>(1).unwrap().quick_assign(move |_seat, _event, _| {
    });


    {
        //
        let egl_display = egl::API.get_display(display_ptr).unwrap();
        egl::API.initialize(egl_display).unwrap();

        let attrs = [
            egl::RED_SIZE  , 8,
            egl::GREEN_SIZE, 8,
            egl::BLUE_SIZE , 8,
            egl::NONE,
        ];

        let config = egl::API.choose_first_config(egl_display, &attrs).unwrap().unwrap();

        let context_attributes = [
            egl::CONTEXT_MAJOR_VERSION, 4,
            egl::CONTEXT_MINOR_VERSION, 0,
            egl::CONTEXT_OPENGL_PROFILE_MASK, egl::CONTEXT_OPENGL_CORE_PROFILE_BIT,
            egl::NONE
        ];

        egl::API.create_context(egl_display, config, None, &context_attributes).unwrap();
    }

    // The compositor allows us to creates surfaces
    // https://wayland-book.com/surfaces/compositor.html
    let compositor = globals.instantiate_exact::<wl_compositor::WlCompositor>(1).unwrap();
    let surface = compositor.create_surface();

    let weak_surface = Arc::downgrade(&surface);

    // The XDG wayland extension is resposible for creating the visible windows
    let xdg_surface = xdg_wm_base.get_xdg_surface(&surface);
    xdg_surface.quick_assign(move |xdg_surface, event, _| match event {
        xdg_surface::Event::Configure { serial } => {
            if let Some(surface) = weak_surface.upcast() {

            }

            xdg_surface.ack_configure(serial);
        }
        _ => unreachable!(),
    });

    let xdg_toplevel = xdg_surface.get_toplevel();
    xdg_toplevel.quick_assign(move |_, event, _| match event {
        xdg_toplevel::Event::Close => {
            exit(0);
        }
        xdg_toplevel::Event::Configure { width: _w, height: _h, .. } => {
        }
        _ => unreachable!(),
    });

    xdg_toplevel.set_title("term".to_string());

    // IDK
    surface.commit();
    event_queue.sync_roundtrip(&mut (), |_, _, _| { /* we ignore unfiltered messages */ }).unwrap();
    surface.commit();

    // Run the main event loop.
    // https://docs.rs/wayland-client/0.28.5/wayland_client/struct.EventQueue.html
    loop {
        event_queue.dispatch(&mut (), |event, _, _| {
            println!("unhandled event: {} - {}", event.interface, event.name);
        }).unwrap();
    }
}
