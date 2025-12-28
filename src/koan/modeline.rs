use std::{ffi::CString, thread, time::Duration};

use super::{
    KoanWM, Window,
    config::{Color, MODELINE_HEIGHT, MODELINE_UPDATE_TIME, MODULES},
    get_pixel_from_color,
};
use x11::xlib;

pub enum ModuleType {
    Clock,
    WindowTitle,
    Custom(fn() -> String),
}

pub struct Module {
    pub kind: ModuleType,
}

impl Module {
    pub fn get_text(&self, focused_title: &str) -> String {
        match self.kind {
            ModuleType::Clock => chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            ModuleType::WindowTitle => focused_title.to_string(),
            ModuleType::Custom(f) => f(),
        }
    }
}

pub struct Modeline {
    width: u32,
    height: u32,
    modules: Vec<Module>,
}

impl KoanWM {
    pub fn create_modeline(&mut self) {
        self.modelines.clear();
        for mon in self.monitors.iter() {
            unsafe {
                let root = xlib::XDefaultRootWindow(self.display);
                let mut attributes: xlib::XSetWindowAttributes = std::mem::zeroed();

                attributes.event_mask = xlib::ExposureMask;
                attributes.background_pixel =
                    get_pixel_from_color(self.display, Color::Secondary.hex());
                attributes.override_redirect = 1;

                let modeline = xlib::XCreateWindow(
                    self.display,
                    root,
                    mon.x,
                    mon.y,
                    mon.width as u32,
                    MODELINE_HEIGHT,
                    0,
                    xlib::CopyFromParent,
                    xlib::InputOutput as u32,
                    std::ptr::null_mut(),
                    xlib::CWBackPixel | xlib::CWEventMask | xlib::CWOverrideRedirect,
                    &mut attributes,
                );

                let gc = xlib::XCreateGC(self.display, modeline, 0, std::ptr::null_mut());
                xlib::XSetForeground(
                    self.display,
                    gc,
                    get_pixel_from_color(self.display, Color::Primary.hex()),
                );

                xlib::XMapWindow(self.display, modeline);
                self.modelines.push(modeline);
                self.gc = gc;
            }
        }
    }

    pub fn update_modeline(&self) {
        if self.modelines.is_empty() {
            return;
        }

        let focused_title = self
            .focused
            .map(|win| self.get_window_title(win))
            .unwrap_or_else(|| "KoanWM".to_string());

        for &modeline in self.modelines.iter() {
            unsafe {
                xlib::XClearWindow(self.display, modeline);
            }
            let mut x_offset = 10;

            for module in MODULES {
                let text = module.get_text(&focused_title);
                let c_text = CString::new(text).unwrap();

                unsafe {
                    xlib::XDrawString(
                        self.display,
                        modeline,
                        self.gc,
                        x_offset,
                        10,
                        c_text.as_ptr(),
                        c_text.to_bytes().len() as i32,
                    );
                }
                x_offset += (c_text.to_bytes().len() as i32 * 8) + 20;
            }
        }
    }

    pub fn spawn_modeline_timer(&self) {
        if self.modelines.is_empty() {
            return;
        }

        let bar_win = self.modelines[0];

        thread::spawn(move || {
            let dpy = unsafe { xlib::XOpenDisplay(std::ptr::null()) };
            if dpy.is_null() {
                return;
            }

            let update_atom =
                unsafe { xlib::XInternAtom(dpy, "MINIWM_UPDATE_BAR\0".as_ptr() as *const _, 0) };

            loop {
                thread::sleep(Duration::from_secs(MODELINE_UPDATE_TIME));
                unsafe {
                    let mut ev: xlib::XClientMessageEvent = std::mem::zeroed();
                    ev.type_ = xlib::ClientMessage;
                    ev.window = bar_win;
                    ev.message_type = update_atom;
                    ev.format = 32;

                    let mut x_ev = xlib::XEvent::from(ev);
                    xlib::XSendEvent(dpy, bar_win, 0, xlib::NoEventMask, &mut x_ev);
                    xlib::XFlush(dpy);
                }
            }
        });
    }

    fn get_window_title(&self, window: Window) -> String {
        unsafe {
            let mut name_ptr: *mut i8 = std::ptr::null_mut();
            if xlib::XFetchName(self.display, window, &mut name_ptr) != 0 && !name_ptr.is_null() {
                let c_str = std::ffi::CStr::from_ptr(name_ptr);
                let title = c_str.to_string_lossy().into_owned();
                xlib::XFree(name_ptr as *mut _);
                title
            } else {
                "No Name".to_string()
            }
        }
    }
}
