use super::{MiniWM, config::AUTO_START, spawn};

impl MiniWM {
    pub fn exec_autostart(&mut self) {
        for app in AUTO_START {
            spawn(app);
        }
    }
}
