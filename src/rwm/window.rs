use x11::xlib;

use crate::rwm::{MiniWM, MiniWMError, BORDER_WIDTH, get_pixel_from_color};

pub type Window = u64;

impl MiniWM {
    pub fn manage_window(&mut self, win: Window) -> Result<(), MiniWMError> {
        self.windows.insert(win);

        self.window_monitors.insert(win, self.current_monitor);

        unsafe {
            xlib::XSelectInput(
                self.display,
                win,
                xlib::EnterWindowMask | xlib::FocusChangeMask,
            );
            xlib::XSetWindowBorderWidth(self.display, win, BORDER_WIDTH);
        }

        self.layout()?;

        unsafe {
            xlib::XMapRaised(self.display, win);
            xlib::XFlush(self.display);
        }

        self.focus_window(win);
        Ok(())
    }

    pub fn focus_window(&mut self, window: Window) {
        if let Some(old_win) = self.focused {
            if old_win != window {
                self.set_border_color(old_win, "#888888");
            }
        }

        if let Some(&mon_idx) = self.window_monitors.get(&window) {
            self.current_monitor = mon_idx;
        }

        self.focused = Some(window);

        unsafe {
            xlib::XSetInputFocus(
                self.display,
                window,
                xlib::RevertToPointerRoot,
                xlib::CurrentTime,
            );
            xlib::XRaiseWindow(self.display, window);
        }

        self.set_border_color(window, "#00ff00");
    }

    pub fn move_window(&self, window: Window, x: i32, y: i32) {
        unsafe { xlib::XMoveWindow(self.display, window, x, y) };
    }

    pub fn resize_window(&self, window: Window, width: u32, height: u32) {
        let safe_width = if width < 1 { 1 } else { width };
        let safe_height = if height < 1 { 1 } else { height };
        unsafe { xlib::XResizeWindow(self.display, window, safe_width, safe_height) };
    }

    pub fn cycle_focus(&mut self, next: bool) {
        let current_windows: Vec<Window> = self
            .windows
            .iter()
            .filter(|w| self.window_monitors.get(w) == Some(&self.current_monitor))
            .copied()
            .collect();

        if current_windows.is_empty() {
            return;
        }

        let target = if let Some(focused) = self.focused {
            if let Some(pos) = current_windows.iter().position(|&w| w == focused) {
                if next {
                    let next_idx = (pos + 1) % current_windows.len();
                    current_windows[next_idx]
                } else {
                    let prev_idx = if pos == 0 {
                        current_windows.len() - 1
                    } else {
                        pos - 1
                    };
                    current_windows[prev_idx]
                }
            } else {
                current_windows[0]
            }
        } else {
            if next {
                current_windows[0]
            } else {
                *current_windows.last().unwrap()
            }
        };

        self.focus_window(target);
    }

    pub fn set_border_color(&self, window: Window, color: &str) {
        let pixel = get_pixel_from_color(self.display, color);
        unsafe {
            xlib::XSetWindowBorder(self.display, window, pixel);
            xlib::XFlush(self.display);
        }
    }

    pub fn send_delete(&mut self, window: Window) {
        unsafe {
            let mut client_ev = xlib::XClientMessageEvent {
                type_: xlib::ClientMessage,
                serial: 0,
                send_event: 1,
                display: self.display,
                window,
                message_type: self.wm_protocols,
                format: 32,
                data: xlib::ClientMessageData::new(),
            };
            client_ev.data.set_long(0, self.wm_delete as i64);
            client_ev.data.set_long(1, xlib::CurrentTime as i64);
            let mut event: xlib::XEvent = client_ev.into();
            xlib::XSendEvent(self.display, window, 0, xlib::NoEventMask, &mut event);
            xlib::XFlush(self.display);
        }
    }
}
