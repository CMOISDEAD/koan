use x11::xlib;

use crate::rwm::{Action, KEY_BINDINGS, MiniWM, MiniWMError, Window};

// ke.state filters
const RELEVANT_MODIFIERS: u32 =
    xlib::ShiftMask
    | xlib::ControlMask
    | xlib::Mod1Mask
    | xlib::Mod4Mask;

impl MiniWM {
    pub fn handle_enter_notify(&mut self, ev: xlib::XEnterWindowEvent) {
        let mouse_x = ev.x_root;
        let mouse_y = ev.y_root;

        for (i, m) in self.monitors.iter().enumerate() {
            if mouse_x >= m.x
                && mouse_x < m.x + m.width
                && mouse_y >= m.y
                && mouse_y < m.y + m.height
            {
                self.current_monitor = i;
                break;
            }
        }

        if self.windows.contains(&ev.window) {
            self.focus_window(ev.window);
        }
    }

    pub fn handle_unmap(&mut self, window: Window) -> Result<(), MiniWMError> {
        let was_focused = self.focused == Some(window);

        let monitor_idx = self.window_monitors.remove(&window).unwrap_or(0);
        self.windows.remove(&window);

        if was_focused {
            self.focused = None;
            let next_focus = self
                .windows
                .iter()
                .filter(|&w| self.window_monitors.get(w) == Some(&monitor_idx))
                .last()
                .copied();

            if let Some(win) = next_focus {
                self.focus_window(win);
            }
        }
        self.layout()?;
        Ok(())
    }

    pub fn handle_keypress(&mut self, ke: xlib::XKeyEvent) -> Result<(), MiniWMError> {
        unsafe {
            let keysym = xlib::XKeycodeToKeysym(self.display, ke.keycode as u8, 0);
            let clean_state = ke.state & RELEVANT_MODIFIERS;
            for binding in KEY_BINDINGS {
                if keysym == binding.keysym as u64
                    && clean_state == binding.modifiers
                {
                    match binding.action {
                        Action::OpenTerminal => {
                            libc::system(b"xterm &\0".as_ptr() as *const i8);
                        }
                        Action::OpenLauncher => {
                            libc::system(b"rofi -show drun &\0".as_ptr() as *const i8);
                        }
                        Action::QuitWM => return Err(MiniWMError::GenericError("Quit".into())),
                        Action::CloseWindow => {
                            if let Some(win) = self.focused {
                                self.send_delete(win);
                            }
                        }
                        Action::FocusPrev => self.cycle_focus(false),
                        Action::FocusNext => self.cycle_focus(true),
                        Action::MonitorPrev => {
                            println!("Focus prev monitor");
                            let _ = self.monitor_prev();
                        }
                        Action::MonitorNext => {
                            println!("Focus next monitor");
                            let _ = self.monitor_next();
                        }
                        Action::MoveWindowToNextMonitor => {
                            println!("Moving window to next monitor");
                            let _ = self.move_window_next_monitor();
                        }
                        Action::MoveWindowToPrevMonitor => {
                            println!("Moving window to prev monitor");
                            let _ = self.move_window_prev_monitor();
                        }
                    }
                }
            }
        }
        Ok(())
    }
}
