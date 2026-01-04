use super::{
    config::{Color, MODELINE_HEIGHT, MODELINE_UPDATE_TIME},
    get_pixel_from_color, KoanWM, Window,
};
use chrono::prelude::*;
use std::{ffi::CString, thread, time::Duration};
use x11::xlib;

pub enum ModuleType {
    Clock,
    WindowTitle,
    ClientCount,
    Custom(fn() -> String),
}

pub enum Alignment {
    Left,
    Center,
    Right,
}

pub struct Module {
    pub kind: ModuleType,
    pub alignment: Alignment,
}

impl Module {
    pub fn get_text(&self, focused_title: &str, client_count: usize) -> String {
        match self.kind {
            ModuleType::Clock => Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            ModuleType::WindowTitle => focused_title.to_string(),
            ModuleType::ClientCount => format!("[{}]", client_count),
            ModuleType::Custom(f) => f(),
        }
    }
}

impl KoanWM {
    pub fn create_modelines(&mut self) {
        self.modelines.clear();
        self.modules = vec![
            Module {
                kind: ModuleType::WindowTitle,
                alignment: Alignment::Left,
            },
            Module {
                kind: ModuleType::ClientCount,
                alignment: Alignment::Left,
            },
            Module {
                kind: ModuleType::Clock,
                alignment: Alignment::Right,
            },
        ];

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

    pub fn update_modelines(&self) {
        if self.modelines.is_empty() {
            return;
        }
        let focused_title = self.get_focus_title();
        let client_count = self.clients.len();

        let char_width: i32 = 8;
        let inter_padding: i32 = 0;
        let margin: i32 = 10;

        let mut left_texts: Vec<String> = vec![];
        let mut center_texts: Vec<String> = vec![];
        let mut right_texts: Vec<String> = vec![];

        for module in &self.modules {
            let text = module.get_text(&focused_title, client_count);
            match module.alignment {
                Alignment::Left => left_texts.push(text),
                Alignment::Center => center_texts.push(text),
                Alignment::Right => right_texts.push(text),
            }
        }

        let compute_width = |texts: &[String]| -> i32 {
            if texts.is_empty() {
                return 0;
            }
            let mut w = 0;
            for (i, t) in texts.iter().enumerate() {
                w += t.len() as i32 * char_width;
                if i < texts.len() - 1 {
                    w += inter_padding;
                }
            }
            w
        };

        let left_w = compute_width(&left_texts);
        let center_w = compute_width(&center_texts);
        let right_w = compute_width(&right_texts);

        for &modeline in self.modelines.iter() {
            let mut root_return: xlib::Window = 0;
            let mut x_return: i32 = 0;
            let mut y_return: i32 = 0;
            let mut width_return: u32 = 0;
            let mut height_return: u32 = 0;
            let mut border_return: u32 = 0;
            let mut depth_return: u32 = 0;
            unsafe {
                xlib::XGetGeometry(
                    self.display,
                    modeline,
                    &mut root_return,
                    &mut x_return,
                    &mut y_return,
                    &mut width_return,
                    &mut height_return,
                    &mut border_return,
                    &mut depth_return,
                );
                xlib::XClearWindow(self.display, modeline);
            }
            let width = width_return as i32;

            // Dibujar izquierda
            let mut x = margin;
            for (i, text) in left_texts.iter().enumerate() {
                if i > 0 {
                    x += inter_padding;
                }
                let c_text = CString::new(text.as_str()).unwrap();
                let len = c_text.to_bytes().len() as i32;
                unsafe {
                    xlib::XDrawString(self.display, modeline, self.gc, x, 10, c_text.as_ptr(), len);
                }
                x += len * char_width;
            }

            // Dibujar derecha
            let right_start = width - margin - right_w;
            x = right_start;
            for (i, text) in right_texts.iter().enumerate() {
                if i > 0 {
                    x += inter_padding;
                }
                let c_text = CString::new(text.as_str()).unwrap();
                let len = c_text.to_bytes().len() as i32;
                unsafe {
                    xlib::XDrawString(self.display, modeline, self.gc, x, 10, c_text.as_ptr(), len);
                }
                x += len * char_width;
            }

            // Dibujar centro
            if !center_texts.is_empty() {
                let avail_start = margin + left_w;
                let avail_end = width - margin - right_w;
                let avail_w = avail_end - avail_start;
                let center_start = avail_start + (avail_w - center_w) / 2;
                x = center_start;
                for (i, text) in center_texts.iter().enumerate() {
                    if i > 0 {
                        x += inter_padding;
                    }
                    let c_text = CString::new(text.as_str()).unwrap();
                    let len = c_text.to_bytes().len() as i32;
                    unsafe {
                        xlib::XDrawString(
                            self.display,
                            modeline,
                            self.gc,
                            x,
                            10,
                            c_text.as_ptr(),
                            len,
                        );
                    }
                    x += len * char_width;
                }
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
}
