use super::{KoanWM, config::AUTO_START, spawn};

impl KoanWM {
    pub fn exec_autostart(&mut self) {
        for app in AUTO_START {
            spawn(app);
        }
    }
}
