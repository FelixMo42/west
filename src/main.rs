// Allow single character names so clippy doesn't lint on x, y, r, g, b, which
// are reasonable variable names in this domain.
#![allow(clippy::many_single_char_names)]

mod vec2;

use vec2::Vec2;

use std::{cmp::min, io::{BufWriter, Write}, os::unix::io::AsRawFd, process::exit};

use wayland_client::{
    event_enum,
    protocol::{wl_compositor, wl_keyboard, wl_surface, wl_pointer, wl_seat, wl_shm},
    Display, Filter, GlobalManager, Main
};
use wayland_protocols::xdg_shell::client::{xdg_surface, xdg_toplevel, xdg_wm_base};
use wayland_egl::WlEglSurface;

fn main() {
    init_wayland_client();
}

fn init_wayland_client() {
    // Create the wl display.
    // https://wayland-book.com/wayland-display.html
    let display = Display::connect_to_env().expect("Failed to create wl display.");

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

    // The compositor allows us to creates surfaces
    // https://wayland-book.com/surfaces/compositor.html
    let compositor = globals.instantiate_exact::<wl_compositor::WlCompositor>(1).unwrap();
    let surface = compositor.create_surface();

    // The SHM represemts the share memory with the server.
    // https://wayland-book.com/surfaces/shared-memory.html
    // let shm = globals.instantiate_exact::<wl_shm::WlShm>(1).unwrap();
 
    // 
    let size = Vec2::new(100, 100);
    let egl_surface = WlEglSurface::new(&surface, size.x, size.y);
    init_gl(egl_surface);

    // The XDG wayland extension is resposible for creating the visible windows
    // https://wayland-book.com/xdg-shell-basics.html
    init_xdg_window(&globals, &surface);

    // Get mouse of keyboard events.
    globals.instantiate_exact::<wl_seat::WlSeat>(1).unwrap().quick_assign(move |seat, event, _| {
    });

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

fn init_gl(surface: WlEglSurface) {
    gl::load_with(|s| surface.ptr());
}

fn init_xdg_window(globals: &GlobalManager, surface: &Main<wl_surface::WlSurface>) {
    let xdg_wm_base = globals
        .instantiate_exact::<xdg_wm_base::XdgWmBase>(2)
        .expect("Compositor does not support xdg_shell");

    xdg_wm_base.quick_assign(|xdg_wm_base, event, _| {
        if let xdg_wm_base::Event::Ping { serial } = event {
            xdg_wm_base.pong(serial);
        };
    });

    let xdg_surface = xdg_wm_base.get_xdg_surface(&surface);
    xdg_surface.quick_assign(move |xdg_surface, event, _| match event {
        xdg_surface::Event::Configure { serial } => {
            xdg_surface.ack_configure(serial);
        }
        _ => unreachable!(),
    });

    let xdg_toplevel = xdg_surface.get_toplevel();
    xdg_toplevel.quick_assign(move |_, event, _| match event {
        xdg_toplevel::Event::Close => {
            exit(0);
        }
        xdg_toplevel::Event::Configure { width, height, .. } => {
        }
        _ => unreachable!(),
    });

    xdg_toplevel.set_title("term".to_string());
}
