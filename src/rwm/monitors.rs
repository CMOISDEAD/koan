use std::{mem::zeroed, slice};

use x11::{xinerama, xlib};

use crate::rwm::MiniWM;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct Monitor {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub index: usize,
}

impl MiniWM {
    pub fn update_monitors(&mut self) {
        unsafe {
            let mut num = 0i32;
            let screens_ptr = xinerama::XineramaQueryScreens(self.display, &mut num);

            self.monitors.clear();

            if screens_ptr.is_null() || num <= 0 {
                let root = xlib::XDefaultRootWindow(self.display);
                let mut root_attr: xlib::XWindowAttributes = zeroed();
                xlib::XGetWindowAttributes(self.display, root, &mut root_attr);
                self.monitors.push(Monitor {
                    x: 0,
                    y: 0,
                    width: root_attr.width,
                    height: root_attr.height,
                    index: 0,
                });
            } else {
                let screens = slice::from_raw_parts(screens_ptr, num as usize);
                for (i, screen) in screens.iter().enumerate() {
                    self.monitors.push(Monitor {
                        x: screen.x_org as i32,
                        y: screen.y_org as i32,
                        width: screen.width as i32,
                        height: screen.height as i32,
                        index: i,
                    });
                }
                xlib::XFree(screens_ptr as *mut _);
            }

            let total_monitors = self.monitors.len();
            for mon_idx in self.window_monitors.values_mut() {
                if *mon_idx >= total_monitors {
                    *mon_idx = 0;
                }
            }

            if self.current_monitor >= total_monitors {
                self.current_monitor = 0;
            }
        }
    }

    pub fn focus_monitor(&mut self, index: usize) {
        let mon = self.monitors[index];
        unsafe {
            xlib::XWarpPointer(
                self.display,
                0, // None
                xlib::XDefaultRootWindow(self.display),
                0,
                0,
                0,
                0,
                mon.x + (mon.width / 2),
                mon.y + (mon.height / 2),
            );
            xlib::XFlush(self.display);
        }

        let last_win = self
            .windows
            .iter()
            .filter(|&w| self.window_monitors.get(w) == Some(&index))
            .last()
            .copied();

        if let Some(win) = last_win {
            self.focus_window(win);
        } else {
            self.focused = None;
        }
    }

    pub fn monitor_next(&mut self) {
        if self.monitors.is_empty() {
            return;
        }

        if self.current_monitor + 1 < self.monitors.len() {
            self.current_monitor += 1;
        }

        self.focus_monitor(self.current_monitor);
    }

    pub fn monitor_prev(&mut self) {
        if self.monitors.is_empty() {
            return;
        }

        if self.current_monitor > 0 {
            self.current_monitor -= 1;
        }

        self.focus_monitor(self.current_monitor);
    }

    pub fn move_window_next_monitor(&mut self) {
        let Some(focused) = self.focused else {
            return;
        };
        let monitor_count = self.monitors.len();
        if monitor_count == 0 {
            return;
        }

        if let Some(idx) = self.window_monitors.get_mut(&focused) {
            if *idx + 1 < monitor_count {
                let new_idx = *idx + 1;
                *idx = new_idx;
                self.focus_monitor(new_idx);
            }
        }

        let _ = self.layout();
    }

    pub fn move_window_prev_monitor(&mut self) {
        let Some(focused) = self.focused else { return };
        let monitor_count = self.monitors.len();
        if monitor_count == 0 {
            return;
        }

        if let Some(idx) = self.window_monitors.get_mut(&focused) {
            if *idx > 0 {
                let new_idx = *idx - 1;
                *idx = new_idx;
                self.focus_monitor(new_idx);
            }
        }

        let _ = self.layout();
    }
}
