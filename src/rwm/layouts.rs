use crate::rwm::{BORDER_WIDTH, MiniWMError, Window, MiniWM};

impl MiniWM {
    pub fn layout(&mut self) -> Result<(), MiniWMError> {
        if self.windows.is_empty() {
            return Ok(());
        }

        for (mon_idx, mon) in self.monitors.iter().enumerate() {
            let monitor_wins: Vec<Window> = self
                .windows
                .iter()
                .filter(|w| self.window_monitors.get(w) == Some(&mon_idx))
                .copied()
                .collect();

            let count = monitor_wins.len();
            if count == 0 {
                continue;
            }

            let win_width = mon.width / count as i32;
            let mut current_x = mon.x;

            for (i, &window) in monitor_wins.iter().enumerate() {
                let width = if i == count - 1 {
                    mon.width - (current_x - mon.x)
                } else {
                    win_width
                };
                let final_w = (width as u32).saturating_sub(BORDER_WIDTH * 2);
                let final_h = (mon.height as u32).saturating_sub(BORDER_WIDTH * 2);

                self.move_window(window, current_x, mon.y);
                self.resize_window(window, final_w, final_h);

                current_x += width;
            }
        }
        Ok(())
    }
}
