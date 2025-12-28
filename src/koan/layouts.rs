use x11::xlib;

use crate::koan::{BORDER_WIDTH, KoanWM, KoanWMError, Window};

use super::config::{GAPS, MODELINE_HEIGHT};

pub enum LAYOUTS {
    MASTER,
    MONOCLE,
}

impl KoanWM {
    pub fn change_split_ratio(&mut self, increase: bool) {
        let step = 0.05;

        if increase {
            self.mfact = (self.mfact + step).min(0.90);
        } else {
            self.mfact = (self.mfact - step).max(0.10);
        }

        if let Err(e) = self.layout() {
            eprintln!("Error al actualizar el layout: {:?}", e);
        }
    }

    pub fn switch_layout(&mut self) {
        self.layout = match self.layout {
            LAYOUTS::MASTER => LAYOUTS::MONOCLE,
            LAYOUTS::MONOCLE => LAYOUTS::MASTER,
        };
        let _ = self.layout();
    }

    pub fn monocle(&mut self, top: i32, bottom: u32) {
        for (mon_idx, mon) in self.monitors.iter().enumerate() {
            let mon_windows: Vec<Window> = self
                .windows
                .iter()
                .filter(|&w| self.window_monitors.get(w) == Some(&mon_idx))
                .copied()
                .collect();

            if mon_windows.is_empty() {
                continue;
            }

            for win in mon_windows {
                self.apply_geometry(
                    win,
                    mon.x + GAPS as i32,
                    mon.y + top,
                    mon.width as u32 - (GAPS * 2),
                    mon.height as u32 - bottom,
                );
            }
        }
    }

    pub fn master_and_stack(&mut self, top: i32, bottom: u32) {
        for (mon_idx, mon) in self.monitors.iter().enumerate() {
            let mon_windows: Vec<Window> = self
                .windows
                .iter()
                .filter(|&w| self.window_monitors.get(w) == Some(&mon_idx))
                .copied()
                .collect();

            let n = mon_windows.len();

            if n == 0 {
                continue;
            }

            if n == 1 {
                self.apply_geometry(
                    mon_windows[0],
                    mon.x + GAPS as i32,
                    mon.y + top,
                    mon.width as u32 - (GAPS * 2),
                    mon.height as u32 - bottom,
                );
            } else {
                let master_w_raw = (mon.width as f32 * self.mfact) as u32;

                self.apply_geometry(
                    mon_windows[0],
                    mon.x + GAPS as i32,
                    mon.y + top,
                    master_w_raw - (GAPS * 2),
                    mon.height as u32 - bottom,
                );

                let stack_x = mon.x + master_w_raw as i32;
                let stack_w_raw = mon.width as u32 - master_w_raw;
                let stack_h_total = mon.height as u32 / (n as u32 - 1);

                for (i, &win) in mon_windows.iter().skip(1).enumerate() {
                    let stack_y = mon.y + (i as i32 * stack_h_total as i32);

                    let height_adjustment = if i == (n - 1) - 1 { bottom } else { GAPS * 2 };

                    self.apply_geometry(
                        win,
                        stack_x + (GAPS / 2) as i32,
                        stack_y + top as i32,
                        stack_w_raw - GAPS - (GAPS / 2),
                        (stack_h_total - height_adjustment) as u32,
                    );
                }
            }
        }
    }

    pub fn layout(&mut self) -> Result<(), KoanWMError> {
        let top = (GAPS + MODELINE_HEIGHT) as i32;
        let bottom = (MODELINE_HEIGHT + GAPS * 2) as u32;

        match self.layout {
            LAYOUTS::MASTER => self.master_and_stack(top, bottom),
            LAYOUTS::MONOCLE => self.monocle(top, bottom),
        }

        Ok(())
    }

    fn apply_geometry(&self, win: Window, x: i32, y: i32, w: u32, h: u32) {
        let final_w = w.saturating_sub(BORDER_WIDTH * 2);
        let final_h = h.saturating_sub(BORDER_WIDTH * 2);

        unsafe {
            xlib::XMoveResizeWindow(self.display, win, x, y, final_w, final_h);
            xlib::XSetWindowBorderWidth(self.display, win, BORDER_WIDTH);
        }
    }
}
