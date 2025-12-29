use super::layouts::LAYOUTS;
use super::monitors::Monitor;
use super::window::{Client, Window};
use super::{config::KEY_BINDINGS, error::KoanWMError};

use std::collections::HashMap;
use std::mem::zeroed;
use std::ptr::null;
use x11::xlib;

extern "C" fn x_error_handler(_: *mut xlib::Display, ev: *mut xlib::XErrorEvent) -> i32 {
    let e = unsafe { *ev };
    if e.error_code == xlib::BadWindow || (e.error_code == 8 && e.request_code == 42) {
        eprintln!(
            "X11 Error: code={}, request={}, minor={}",
            e.error_code, e.request_code, e.minor_code
        );
        return 0;
    }
    0
}

pub struct KoanWM {
    pub mfact: f32,
    pub layout: LAYOUTS,
    pub display: *mut xlib::Display,

    pub monitors: Vec<Monitor>,
    pub current_monitor: usize,

    pub clients: Vec<Client>,
    pub focused: Option<Window>,
    pub window_monitors: HashMap<Window, usize>, // window, monitor_idx

    pub gc: xlib::GC, // Contexto gráfico para dibujar
    pub modelines: Vec<xlib::Window>,

    // atoms
    pub wm_protocols: xlib::Atom,
    pub wm_delete: xlib::Atom,
    pub wm_state: xlib::Atom,
    pub wm_state_above: xlib::Atom,
}

impl KoanWM {
    pub fn new() -> Result<Self, KoanWMError> {
        unsafe {
            let display = xlib::XOpenDisplay(null());
            if display.is_null() {
                return Err(KoanWMError::DisplayNotFound);
            }

            xlib::XSetErrorHandler(Some(x_error_handler));

            let wm_protocols = xlib::XInternAtom(display, "WM_PROTOCOLS\0".as_ptr() as *const _, 0);
            let wm_delete =
                xlib::XInternAtom(display, "WM_DELETE_WINDOW\0".as_ptr() as *const _, 0);
            let wm_state = xlib::XInternAtom(display, "_NET_WM_STATE\0".as_ptr() as *const _, 0);
            let wm_state_above =
                xlib::XInternAtom(display, "_NET_WM_STATE_ABOVE\0".as_ptr() as *const _, 0);

            Ok(Self {
                layout: LAYOUTS::MONOCLE,
                mfact: 0.5,
                display,
                clients: Vec::new(),
                window_monitors: HashMap::new(),
                focused: None,
                monitors: Vec::new(),
                current_monitor: 0,
                modelines: Vec::new(),
                gc: std::ptr::null_mut(),
                wm_protocols,
                wm_delete,
                wm_state,
                wm_state_above,
            })
        }
    }

    pub fn init(&mut self) -> Result<(), KoanWMError> {
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
            self.create_modeline();
            self.spawn_modeline_timer();

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

    pub fn run(&mut self) -> Result<(), KoanWMError> {
        let mut event: xlib::XEvent = unsafe { zeroed() };
        loop {
            unsafe {
                xlib::XNextEvent(self.display, &mut event);
                match event.get_type() {
                    xlib::ClientMessage => {
                        let ev = xlib::XClientMessageEvent::from(event);
                        let update_atom = xlib::XInternAtom(
                            self.display,
                            "MINIWM_UPDATE_BAR\0".as_ptr() as *const _,
                            0,
                        );

                        if ev.message_type == update_atom {
                            self.update_modeline();
                        }
                    }
                    xlib::Expose => {
                        let ev = xlib::XExposeEvent::from(event);
                        if self.modelines.contains(&ev.window) {
                            self.update_modeline();
                        }
                    }
                    xlib::ConfigureNotify => {
                        let ev = xlib::XConfigureEvent::from(event);
                        let root = xlib::XDefaultRootWindow(self.display);
                        if ev.window == root {
                            self.create_modeline();
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
