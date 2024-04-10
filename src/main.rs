mod ffi;
use ffi as c;

// we import the necessary modules (only the core X module in this application).
use vi::telex::transform_buffer;
use xcb::x;

fn main() -> xcb::Result<()> {
    // Connect to the X server.
    let (conn, screen_id) = xcb::Connection::connect(None)?;

    // Fetch the `x::Setup` and get the main `x::Screen` object.
    let setup = conn.get_setup();
    let screen = setup.roots().nth(screen_id as usize).unwrap();

    eprintln!(
        "Screen dimensions: {w}, {h}",
        w = screen.width_in_pixels(),
        h = screen.height_in_pixels()
    );

    // Generate an `Xid` for the client window.
    // The type inference is needed here.
    let window: x::Window = conn.generate_id();

    // We can now create a window. For this we pass a `Request`
    // object to the `send_request_checked` method. The method
    // returns a cookie that will be used to check for success.
    let cookie = conn.send_request_checked(&x::CreateWindow {
        depth: x::COPY_FROM_PARENT as u8,
        wid: window,
        parent: screen.root(),
        x: 0,
        y: 0,
        width: 150,
        height: 150,
        border_width: 0,
        class: x::WindowClass::InputOutput,
        visual: screen.root_visual(),
        // this list must be in same order than `Cw` enum order
        value_list: &[
            x::Cw::BackPixel(screen.white_pixel()),
            x::Cw::EventMask(x::EventMask::EXPOSURE | x::EventMask::KEY_PRESS),
        ],
    });

    // We now check if the window creation worked.
    // A cookie can't be cloned; it is moved to the function.
    conn.check_request(cookie)?;

    // Let's change the window title
    let cookie = conn.send_request_checked(&x::ChangeProperty {
        mode: x::PropMode::Replace,
        window,
        property: x::ATOM_WM_NAME,
        r#type: x::ATOM_STRING,
        data: b"My XCB Window",
    });
    // And check for success again
    conn.check_request(cookie)?;

    // We now show ("map" in X terminology) the window.
    // This time we do not check for success, so we discard the cookie.
    conn.send_request(&x::MapWindow { window });

    // Previous request was checked, so a flush is not necessary in this case.
    // Otherwise, here is how to perform a connection flush.
    conn.flush()?;

    let mut buf: Vec<char> = Vec::with_capacity(32);
    let mut out = String::with_capacity(32);

    #[repr(u32)]
    enum Col {
        Lower = 0,
        Upper = 1,
    }

    let key_symbols = unsafe { c::xcb_key_symbols_alloc(conn.get_raw_conn().cast()) };

    // proceed with create_window and event loop...
    // We enter the main event loop
    loop {
        match conn.wait_for_event()? {
            xcb::Event::X(x::Event::Expose(expose)) => {
                eprintln!("{:?} exposed. Region to be redrawn at location ({},{}), with dimension ({},{})",
                    expose.window(), expose.x(), expose.y(), expose.width(), expose.height());
            }
            xcb::Event::X(x::Event::KeyRelease(ev)) => {
                eprintln!("Last key released: code (0x:{:04x}", ev.detail());
                continue;
            }
            xcb::Event::X(x::Event::KeyPress(ev)) => {
                let kcode = ev.detail();
                dbg!(kcode);
                let col = {
                    let shift_on = ev.state().contains(x::KeyButMask::SHIFT);
                    let caps_on = ev.state().contains(x::KeyButMask::LOCK);
                    match (shift_on, caps_on) {
                        (true, true) | (false, false) => Col::Lower,
                        _other => Col::Upper,
                    }
                };

                let ksym = unsafe { c::xcb_key_symbols_get_keysym(key_symbols, kcode, col as _) };

                match ksym {
                    // Latin-1 characters have the same representation.
                    c::XK_A..=c::XK_Z | c::XK_a..=c::XK_z => {
                        let c = char::from(ksym as u8);
                        buf.push(c);
                    }
                    c::XK_space | c::XK_Return | 0 => {
                        transform_buffer(buf.clone(), &mut out);
                        dbg!(&out);
                        buf.clear();
                        out.clear();
                    }
                    c::XK_BackSpace => {
                        buf.pop();
                    }
                    other => {
                        eprintln!("ksym = 0x{:04X}", other);
                    }
                }
            }
            xcb::Event::X(x::Event::ClientMessage(ev)) => {
                // We have received a message from the server
                if let x::ClientMessageData::Data32([atom, ..]) = ev.data() {
                    dbg!(atom);
                }
            }
            xcb::Event::X(ev) => {
                dbg!(ev);
            }
            other => {
                dbg!(other);
                break;
            }
        }
    }
    unsafe {
        c::xcb_key_symbols_free(key_symbols);
    }
    Ok(())
}
