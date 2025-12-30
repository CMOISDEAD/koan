use x11::xlib;

use super::{
    config::{CURSOR_ENTER_FOCUS, LAUNCHER, TERMINAL},
    utils::spawn,
    window::Direction,
    Action, KoanWM, KoanWMError, Window, KEY_BINDINGS,
};

// ke.state filters
const RELEVANT_MODIFIERS: u32 =
    xlib::ShiftMask | xlib::ControlMask | xlib::Mod1Mask | xlib::Mod4Mask;

impl KoanWM {
    pub fn handle_enter_notify(&mut self, ev: xlib::XEnterWindowEvent) {
        if !CURSOR_ENTER_FOCUS {
            return;
        }

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

        if self.clients.iter().any(|c| c.window == ev.window) {
            self.focus_window(ev.window);
        }
    }

    pub fn handle_unmap(&mut self, window: Window) -> Result<(), KoanWMError> {
        let client = match self.clients.iter().find(|c| c.window == window) {
            Some(c) => c,
            None => return Ok(()),
        };

        let was_focused = self.focused == Some(client.window);

        let monitor_idx = self.window_monitors.remove(&client.window).unwrap_or(0);
        self.clients.retain(|c| c.window != window);

        if was_focused {
            self.focused = None;
            let next_focus = self
                .clients
                .iter()
                .filter(|&w| self.window_monitors.get(&w.window) == Some(&monitor_idx))
                .last();

            if let Some(client) = next_focus {
                self.focus_window(client.window);
            }
        }
        self.layout()?;
        Ok(())
    }

    pub fn handle_keypress(&mut self, ke: xlib::XKeyEvent) -> Result<(), KoanWMError> {
        unsafe {
            let keysym = xlib::XKeycodeToKeysym(self.display, ke.keycode as u8, 0);
            let clean_state = ke.state & RELEVANT_MODIFIERS;
            for binding in KEY_BINDINGS {
                if keysym == binding.keysym as u64 && clean_state == binding.modifiers {
                    match binding.action {
                        Action::SwitchLayout => {
                            self.switch_layout();
                        }
                        Action::OpenTerminal => {
                            spawn(&TERMINAL);
                        }
                        Action::OpenLauncher => {
                            spawn(&LAUNCHER);
                        }
                        Action::QuitWM => return Err(KoanWMError::GenericError("Quit".into())),
                        Action::CloseWindow => {
                            if let Some(win) = self.focused {
                                self.send_delete(win)?;
                            }
                        }
                        Action::FocusUp => {
                            println!("up")
                        }
                        Action::FocusDown => {
                            println!("down")
                        }
                        Action::FocusRight => self.cycle_focus(true),
                        Action::FocusLeft => self.cycle_focus(false),
                        Action::MonitorPrev => {
                            let _ = self.monitor_prev();
                        }
                        Action::MonitorNext => {
                            let _ = self.monitor_next();
                        }
                        Action::MoveWindowToNextMonitor => {
                            let _ = self.move_window_next_monitor();
                        }
                        Action::MoveWindowToPrevMonitor => {
                            let _ = self.move_window_prev_monitor();
                        }
                        Action::MoveUp => {
                            let client = match self.focused_client() {
                                Some(c) => c,
                                None => return Ok(()),
                            };

                            if client.is_float {
                                self.move_window(Direction::Up);
                            } else {
                                // self.swap_window(true);
                            }
                        }
                        Action::MoveDown => {
                            let client = match self.focused_client() {
                                Some(c) => c,
                                None => return Ok(()),
                            };

                            if client.is_float {
                                self.move_window(Direction::Down);
                            } else {
                                // self.swap_window(true);
                            }
                        }
                        Action::MoveRight => {
                            let client = match self.focused_client() {
                                Some(c) => c,
                                None => return Ok(()),
                            };

                            if client.is_float {
                                self.move_window(Direction::Right);
                            } else {
                                self.swap_window(true);
                            }
                        }
                        Action::MoveLeft => {
                            let client = match self.focused_client() {
                                Some(c) => c,
                                None => return Ok(()),
                            };

                            if client.is_float {
                                self.move_window(Direction::Left);
                            } else {
                                self.swap_window(false);
                            }
                        }
                        Action::IncreaseHeight => {
                            let client = match self.focused_client() {
                                Some(c) => c,
                                None => return Ok(()),
                            };

                            if client.is_float {
                                self.resize_window(Direction::Down);
                            } else {
                                self.change_split_ratio(true);
                            }
                        }
                        Action::DecreaseHeight => {
                            let client = match self.focused_client() {
                                Some(c) => c,
                                None => return Ok(()),
                            };

                            if client.is_float {
                                self.resize_window(Direction::Up);
                            } else {
                                self.change_split_ratio(false);
                            }
                        }
                        Action::IncreaseWidth => {
                            if let Some(client) = self.focused_client() {
                                if client.is_float {
                                    self.resize_window(Direction::Right);
                                }
                            }
                        }
                        Action::DecreaseWidth => {
                            if let Some(client) = self.focused_client() {
                                if client.is_float {
                                    self.resize_window(Direction::Left);
                                }
                            }
                        }
                        Action::ToggleFloat => {
                            self.toggle_float();
                        }
                    }
                }
            }
        }
        Ok(())
    }
}
