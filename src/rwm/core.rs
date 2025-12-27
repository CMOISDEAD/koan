use super::monitors::Monitor;
use super::window::Window;
use super::{config::KEY_BINDINGS, error::MiniWMError};

use std::collections::{BTreeSet, HashMap};
use std::mem::zeroed;
use std::ptr::null;
use x11::xlib;

extern "C" fn x_error_handler(_: *mut xlib::Display, ev: *mut xlib::XErrorEvent) -> i32 {
    let e = unsafe { *ev };
    if e.error_code == xlib::BadWindow || (e.error_code == 8 && e.request_code == 42) {
        return 0;
    }
    0
}

pub struct MiniWM {
    pub monitors: Vec<Monitor>,
    pub current_monitor: usize,
    pub display: *mut xlib::Display,

    pub windows: BTreeSet<Window>,
    pub window_monitors: HashMap<Window, usize>, // window, monitor_idx

    pub focused: Option<Window>,

    // atoms
    pub wm_protocols: xlib::Atom,
    pub wm_delete: xlib::Atom,
}

impl MiniWM {
    pub fn new() -> Result<Self, MiniWMError> {
        unsafe {
            let display = xlib::XOpenDisplay(null());
            if display.is_null() {
                return Err(MiniWMError::DisplayNotFound);
            }

            xlib::XSetErrorHandler(Some(x_error_handler));

            let wm_protocols = xlib::XInternAtom(display, "WM_PROTOCOLS\0".as_ptr() as *const _, 0);
            let wm_delete =
                xlib::XInternAtom(display, "WM_DELETE_WINDOW\0".as_ptr() as *const _, 0);

            Ok(Self {
                display,
                windows: BTreeSet::new(),
                window_monitors: HashMap::new(),
                focused: None,
                monitors: Vec::new(),
                current_monitor: 0,
                wm_protocols,
                wm_delete,
            })
        }
    }

    pub fn init(&mut self) -> Result<(), MiniWMError> {
        unsafe {
            let root = xlib::XDefaultRootWindow(self.display);
            xlib::XSelectInput(
                self.display,
                root,
                xlib::SubstructureRedirectMask
                    | xlib::SubstructureNotifyMask
                    | xlib::FocusChangeMask
                    | xlib::EnterWindowMask
                    | xlib::StructureNotifyMask,
            );

            self.update_monitors();
            self.exec_autostart();

            xlib::XUngrabKey(self.display, xlib::AnyKey, xlib::AnyModifier, root);

            for binding in KEY_BINDINGS {
                let keycode = xlib::XKeysymToKeycode(self.display, binding.keysym as u64);
                let status = xlib::XGrabKey(
                    self.display,
                    keycode as i32,
                    binding.modifiers,
                    root,
                    0,
                    xlib::GrabModeAsync,
                    xlib::GrabModeAsync,
                );
                if status == 0 {
                    eprintln!("Warning: XGrabKey failed for keysym {}", binding.keysym);
                }
            }
        }
        Ok(())
    }

    pub fn run(&mut self) -> Result<(), MiniWMError> {
        let mut event: xlib::XEvent = unsafe { zeroed() };
        loop {
            unsafe {
                xlib::XNextEvent(self.display, &mut event);
                match event.get_type() {
                    xlib::ConfigureNotify => {
                        let ev = xlib::XConfigureEvent::from(event);
                        let root = xlib::XDefaultRootWindow(self.display);

                        if ev.window == root {
                            self.update_monitors();
                            self.layout()?;
                        }
                    }
                    xlib::MapRequest => {
                        let ev = xlib::XMapRequestEvent::from(event);
                        self.manage_window(ev.window)?;
                    }
                    xlib::UnmapNotify => {
                        let ev = xlib::XUnmapEvent::from(event);
                        self.handle_unmap(ev.window)?;
                    }
                    xlib::EnterNotify => {
                        let ev = xlib::XEnterWindowEvent::from(event);
                        self.handle_enter_notify(ev);
                    }
                    xlib::KeyPress => {
                        if let Err(_) = self.handle_keypress(xlib::XKeyEvent::from(event)) {
                            return Ok(());
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}
