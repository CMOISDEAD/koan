use x11::xlib;

use crate::koan::{BORDER_WIDTH, KoanWM, KoanWMError, get_pixel_from_color};

use super::{config::Color, layouts::LAYOUTS};

pub type Window = u64;

impl KoanWM {
    pub fn manage_window(&mut self, win: Window) -> Result<(), KoanWMError> {
        unsafe {
            let mut transient_for: xlib::Window = 0;
            if xlib::XGetTransientForHint(self.display, win, &mut transient_for) != 0 {
                xlib::XMapRaised(self.display, win);
                self.center_window(win);
                return Ok(());
            }
        }

        match self.layout {
            LAYOUTS::MASTER => self.windows.push(win),
            LAYOUTS::MONOCLE=> self.windows.insert(0, win),
        }

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

        // Obtenemos los índices de las ventanas del monitor actual
        let monitor_indices: Vec<usize> = self
            .windows
            .iter()
            .enumerate()
            .filter(|&(_, &w)| self.window_monitors.get(&w) == Some(&self.current_monitor))
            .map(|(i, _)| i)
            .collect();

        if let Some(pos_in_filter) = monitor_indices
            .iter()
            .position(|&idx| self.windows[idx] == focused_win)
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

                self.windows.swap(idx_a, idx_b);
                let _ = self.layout(); // Al intercambiar en el Vec, el layout las dibujará cambiadas
            }
        }
    }

    pub fn _move_window(&self, window: Window, x: i32, y: i32) {
        unsafe { xlib::XMoveWindow(self.display, window, x, y) };
    }

    pub fn _resize_window(&mut self, increase: bool) {
        // 1. Obtenemos la ventana enfocada
        let focused = match self.focused {
            Some(w) => w,
            None => return,
        };

        // 2. Comprobamos si la ventana es flotante (lógica futura)
        // Por ahora asumimos que todo es tiling (false)
        let is_floating = false;

        if is_floating {
            // LÓGICA FLOTANTE (Tu código antiguo corregido)
            if let Some(attr) = self.get_window_geometry(focused) {
                let change = if increase { 20 } else { -20 };
                let new_w = (attr.width as i32 + change).max(50) as u32; // Mínimo 50px
                let new_h = (attr.height as i32 + change).max(50) as u32;

                unsafe {
                    xlib::XResizeWindow(self.display, focused, new_w, new_h);
                }
            }
        } else {
            // LÓGICA TILING (La correcta para tu layout actual)
            // En tiling, "redimensionar" significa mover la división del Master
            self.change_split_ratio(increase);
        }
    }

    fn center_window(&self, win: Window) {
        let mon = self.monitors[self.current_monitor];
        let mut attr: xlib::XWindowAttributes = unsafe { std::mem::zeroed() };

        unsafe {
            xlib::XGetWindowAttributes(self.display, win, &mut attr);
            let x = mon.x + (mon.width - attr.width) / 2;
            let y = mon.y + (mon.height - attr.height) / 2;
            xlib::XMoveWindow(self.display, win, x, y);
        }
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
                    if pos + 1 < current_windows.len() {
                        Some(current_windows[pos + 1])
                    } else {
                        None
                    }
                } else {
                    if pos > 0 {
                        Some(current_windows[pos - 1])
                    } else {
                        None
                    }
                }
            } else {
                Some(current_windows[0])
            }
        } else {
            Some(current_windows[0])
        };

        if let Some(win) = target {
            self.focus_window(win);
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
        let monitor_windows: Vec<xlib::Window> = self
            .windows
            .iter()
            .filter(|&&w| self.window_monitors.get(&w) == Some(&self.current_monitor))
            .copied()
            .collect();

        if monitor_windows.is_empty() {
            return None;
        }

        let current_focused = match self.focused {
            Some(f) => f,
            None => return Some(monitor_windows[0]),
        };

        let pos = monitor_windows.iter().position(|&w| w == current_focused)?;

        if next {
            if pos + 1 < monitor_windows.len() {
                Some(monitor_windows[pos + 1])
            } else {
                None
            }
        } else {
            if pos > 0 {
                Some(monitor_windows[pos - 1])
            } else {
                None
            }
        }
    }

    pub fn get_window_geometry(&self, window: xlib::Window) -> Option<xlib::XWindowAttributes> {
        unsafe {
            let mut attrs: xlib::XWindowAttributes = std::mem::zeroed();
            if xlib::XGetWindowAttributes(self.display, window, &mut attrs) != 0 {
                Some(attrs)
            } else {
                None
            }
        }
    }
}
