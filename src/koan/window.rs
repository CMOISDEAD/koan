use x11::xlib;

use crate::koan::{get_pixel_from_color, KoanWM, KoanWMError, BORDER_WIDTH};

use super::{config::Color, layouts::LAYOUTS};

pub type Window = u64;

pub struct Client {
    pub window: Window,
    pub is_float: bool,
}

impl KoanWM {
    fn should_manage_window(&self, window: Window) -> bool {
        unsafe {
            if self.clients.iter().any(|c| c.window == window) {
                eprintln!("Window {} already managed", window);
                return false;
            }

            let mut wa: xlib::XWindowAttributes = std::mem::zeroed();
            if xlib::XGetWindowAttributes(self.display, window, &mut wa) == 0 {
                eprintln!("Failed to get window attributes for {}", window);
                return false;
            }

            if wa.override_redirect != 0 {
                eprintln!("Window {} is override-redirect, ignoring", window);
                return false;
            }

            if self.is_auxiliary_window_type(window) {
                eprintln!("Window {} is auxiliary type, ignoring", window);
                return false;
            }

            if wa.width < 1 || wa.height < 1 {
                eprintln!(
                    "Window {} has dimensions {}x{}, possibly auxiliary",
                    window, wa.width, wa.height
                );
                return false;
            }

            eprintln!("Window {} passed all checks, managing it", window);
            true
        }
    }

    fn is_auxiliary_window_type(&self, window: Window) -> bool {
        unsafe {
            let net_wm_window_type = xlib::XInternAtom(
                self.display,
                b"_NET_WM_WINDOW_TYPE\0".as_ptr() as *const i8,
                xlib::False,
            );

            let mut actual_type: xlib::Atom = 0;
            let mut actual_format: i32 = 0;
            let mut nitems: u64 = 0;
            let mut bytes_after: u64 = 0;
            let mut prop: *mut u8 = std::ptr::null_mut();

            let result = xlib::XGetWindowProperty(
                self.display,
                window,
                net_wm_window_type,
                0,
                1,
                xlib::False,
                xlib::XA_ATOM,
                &mut actual_type,
                &mut actual_format,
                &mut nitems,
                &mut bytes_after,
                &mut prop,
            );

            if result != 0 || prop.is_null() || nitems == 0 {
                return false;
            }

            let window_type = *(prop as *const xlib::Atom);
            xlib::XFree(prop as *mut std::ffi::c_void);

            let ignore_types: &[&[u8]] = &[
                b"_NET_WM_WINDOW_TYPE_SPLASH\0",
                b"_NET_WM_WINDOW_TYPE_TOOLBAR\0",
                b"_NET_WM_WINDOW_TYPE_MENU\0",
                b"_NET_WM_WINDOW_TYPE_DROPDOWN_MENU\0",
                b"_NET_WM_WINDOW_TYPE_POPUP_MENU\0",
                b"_NET_WM_WINDOW_TYPE_TOOLTIP\0",
                b"_NET_WM_WINDOW_TYPE_NOTIFICATION\0",
                b"_NET_WM_WINDOW_TYPE_COMBO\0",
                b"_NET_WM_WINDOW_TYPE_DND\0",
            ];

            for ignore_type in ignore_types {
                let atom =
                    xlib::XInternAtom(self.display, ignore_type.as_ptr() as *const i8, xlib::False);
                if window_type == atom {
                    eprintln!("Window {} has auxiliary type", window);
                    return true;
                }
            }

            false
        }
    }

    pub fn manage_window(&mut self, window: Window) -> Result<(), KoanWMError> {
        eprintln!("=== Attempting to manage window {} ===", window);

        unsafe {
            if !self.should_manage_window(window) {
                eprintln!("Window {} rejected by rules", window);
                return Ok(());
            }

            eprintln!("Managing window {}", window);

            let mut client: Client = Client {
                window,
                is_float: false,
            };

            let mut transient_for: xlib::Window = 0;
            if xlib::XGetTransientForHint(self.display, window, &mut transient_for) != 0 {
                eprintln!("Window {} is transient", window);
                xlib::XMapRaised(self.display, window);
                xlib::XChangeProperty(
                    self.display,
                    window,
                    self.wm_state,
                    xlib::XA_ATOM,
                    32,
                    xlib::PropModeAppend,
                    &self.wm_state_above as *const xlib::Atom as *const u8,
                    1,
                );
                client.is_float = true;
                self.center_window(client.window);
                self.window_monitors
                    .insert(client.window, self.current_monitor);
                self.clients.push(client);
                return Ok(());
            }

            match self.layout {
                LAYOUTS::MASTER => self.clients.push(client),
                LAYOUTS::MONOCLE => self.clients.insert(0, client),
            }
            self.window_monitors.insert(window, self.current_monitor);

            xlib::XSelectInput(
                self.display,
                window,
                xlib::EnterWindowMask | xlib::FocusChangeMask,
            );
            xlib::XSetWindowBorderWidth(self.display, window, BORDER_WIDTH);

            self.layout()?;
            xlib::XMapRaised(self.display, window);
            xlib::XFlush(self.display);
            self.focus_window(window);

            eprintln!("Successfully managed window {}", window);
        }

        Ok(())
    }

    pub fn focus_window(&mut self, window: Window) {
        if let Some(old_win) = self.focused {
            if old_win != window {
                self.set_border_color(old_win, Color::Secondary.hex());
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

        self.set_border_color(window, Color::Primary.hex());
        self.update_modeline();
    }

    pub fn swap_window(&mut self, next: bool) {
        let focused_win = match self.focused {
            Some(w) => w,
            None => return,
        };

        let monitor_indices: Vec<usize> = self
            .clients
            .iter()
            .enumerate()
            .filter(|&(_, c)| self.window_monitors.get(&c.window) == Some(&self.current_monitor))
            .map(|(i, _)| i)
            .collect();

        if let Some(pos_in_filter) = monitor_indices
            .iter()
            .position(|&idx| self.clients[idx].window == focused_win)
        {
            let target_filter_idx = if next {
                if pos_in_filter + 1 < monitor_indices.len() {
                    Some(pos_in_filter + 1)
                } else {
                    None
                }
            } else {
                if pos_in_filter > 0 {
                    Some(pos_in_filter - 1)
                } else {
                    None
                }
            };

            if let Some(t_idx) = target_filter_idx {
                let idx_a = monitor_indices[pos_in_filter];
                let idx_b = monitor_indices[t_idx];

                self.clients.swap(idx_a, idx_b);
                let _ = self.layout();
            }
        }
    }

    pub fn _move_window(&self, window: Window, x: i32, y: i32) {
        unsafe { xlib::XMoveWindow(self.display, window, x, y) };
    }

    pub fn _resize_window(&mut self, increase: bool) {
        let focused = match self.focused {
            Some(w) => w,
            None => return,
        };

        let client = match self.clients.iter().find(|c| c.window == focused) {
            Some(c) => c,
            None => return,
        };

        if client.is_float {
            if let Some(attr) = self._get_window_geometry(focused) {
                let change = if increase { 20 } else { -20 };
                let new_w = (attr.width as i32 + change).max(50) as u32;
                let new_h = (attr.height as i32 + change).max(50) as u32;

                unsafe {
                    xlib::XResizeWindow(self.display, focused, new_w, new_h);
                }
            }
        } else {
            self.change_split_ratio(increase);
        }
    }

    pub fn toggle_float(&mut self) {
        let focused = match self.focused {
            Some(w) => w,
            None => return,
        };

        if let Some(client) = self.clients.iter_mut().find(|c| c.window == focused) {
            client.is_float = !client.is_float;
        }

        let _ = self.layout();
    }

    pub fn center_window(&self, window: Window) {
        let mon = self.monitors[self.current_monitor];
        let mut attr: xlib::XWindowAttributes = unsafe { std::mem::zeroed() };

        unsafe {
            xlib::XGetWindowAttributes(self.display, window, &mut attr);
            let x = mon.x + (mon.width - attr.width) / 2;
            let y = mon.y + (mon.height - attr.height) / 2;
            xlib::XMoveWindow(self.display, window, x, y);
        }
    }

    pub fn cycle_focus(&mut self, next: bool) {
        let current_client: Vec<&Client> = self
            .clients
            .iter()
            .filter(|c| self.window_monitors.get(&c.window) == Some(&self.current_monitor))
            .collect();

        if current_client.is_empty() {
            return;
        }

        let target = if let Some(focused) = self.focused {
            if let Some(pos) = current_client.iter().position(|&c| c.window == focused) {
                if next {
                    if pos + 1 < current_client.len() {
                        Some(current_client[pos + 1])
                    } else {
                        None
                    }
                } else {
                    if pos > 0 {
                        Some(current_client[pos - 1])
                    } else {
                        None
                    }
                }
            } else {
                Some(current_client[0])
            }
        } else {
            Some(current_client[0])
        };

        if let Some(client) = target {
            self.focus_window(client.window);
        }
    }

    pub fn set_border_color(&self, window: Window, color: &str) {
        let pixel = get_pixel_from_color(self.display, color);
        unsafe {
            xlib::XSetWindowBorder(self.display, window, pixel);
            xlib::XFlush(self.display);
        }
    }

    pub fn send_delete(&mut self, window: Window) -> Result<(), KoanWMError> {
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
            Ok(())
        }
    }

    pub fn _get_adjacent_window(&self, next: bool) -> Option<xlib::Window> {
        let monitor_windows: Vec<&Client> = self
            .clients
            .iter()
            .filter(|&c| self.window_monitors.get(&c.window) == Some(&self.current_monitor))
            .collect();

        if monitor_windows.is_empty() {
            return None;
        }

        let current_focused = match self.focused {
            Some(f) => f,
            None => return Some(monitor_windows[0].window),
        };

        let pos = monitor_windows
            .iter()
            .position(|&c| c.window == current_focused)?;

        if next {
            if pos + 1 < monitor_windows.len() {
                Some(monitor_windows[pos + 1].window)
            } else {
                None
            }
        } else {
            if pos > 0 {
                Some(monitor_windows[pos - 1].window)
            } else {
                None
            }
        }
    }

    pub fn _get_window_geometry(&self, window: xlib::Window) -> Option<xlib::XWindowAttributes> {
        unsafe {
            let mut attrs: xlib::XWindowAttributes = std::mem::zeroed();
            if xlib::XGetWindowAttributes(self.display, window, &mut attrs) != 0 {
                Some(attrs)
            } else {
                None
            }
        }
    }

    pub fn _is_floating(&mut self, _window: Window) -> bool {
        true
        // if xlib::XGetWindowProperty(self.display, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1)
    }
}
