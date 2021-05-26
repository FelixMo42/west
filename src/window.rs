use wayland_client::EventQueue;
use wayland_client::{protocol::wl_compositor::WlCompositor, Display, Main};
use wayland_protocols::xdg_shell::client::xdg_wm_base;

pub struct DisplayConnection {
    pub display: Display,
    pub event_queue: EventQueue,
    pub compositor: Main<WlCompositor>,
    pub xdg: Main<xdg_wm_base::XdgWmBase>,
}

pub fn setup_wayland() -> DisplayConnection {
    // Display is the entry point to the wayland protocol.
    // https://wayland-book.com/wayland-display.html
    let display =
        wayland_client::Display::connect_to_env().expect("unable to connect to the wayland server");

    // Create an event queue for messages to and from the wl server and attach it.
    // https://docs.rs/wayland-client/0.28.5/wayland_client/struct.EventQueue.html
    let mut event_queue = display.create_event_queue();
    let attached_display = display.clone().attach(event_queue.token());

    // Globals are the list of objects in the connection, and its used to make new ones.
    // https://wayland-book.com/registry.html
    let globals = wayland_client::GlobalManager::new(&attached_display);

    // Sync with the server to make sure we have the globals list.
    event_queue
        .sync_roundtrip(&mut (), |_, _, _| unreachable!())
        .unwrap();

    // Get the compositor. Resposible for managing pixel buffers.
    // https://wayland-book.com/surfaces/compositor.html
    let compositor: Main<WlCompositor> = globals.instantiate_exact(1).unwrap();

    // Xdg protocol is a wayland extension that describes application windows.
    // Xdg_base is the parts of the protocol shared bettween the windows.
    // https://wayland-book.com/xdg-shell-basics.html
    let xdg = globals
        .instantiate_exact::<xdg_wm_base::XdgWmBase>(1)
        .unwrap();

    // The only part we need to implement is ping, so that the server knows if we're responsive.
    xdg.quick_assign(|xdg, event, _| {
        if let xdg_wm_base::Event::Ping { serial } = event {
            xdg.pong(serial);
        }
    });

    // Return the important parts of the connect
    DisplayConnection {
        display,
        event_queue,
        compositor,
        xdg,
    }
}

impl DisplayConnection {
    pub fn sync(&mut self) {
        self.event_queue
            .sync_roundtrip(&mut (), |_, _, _| unreachable!())
            .unwrap();
    }
}
