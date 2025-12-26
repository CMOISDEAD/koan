use x11::keysym::{XK_J, XK_K, XK_Return, XK_c, XK_d, XK_i, XK_j, XK_k, XK_o, XK_q};
use x11::xlib::Mod1Mask;

#[derive(Debug, Clone, Copy)]
pub enum Action {
    OpenLauncher,
    OpenTerminal,
    QuitWM,
    FocusNext,
    FocusPrev,
    MonitorNext,
    MonitorPrev,
    CloseWindow,
}

const MOD_KEY: u32 = Mod1Mask;

pub struct KeyBinding {
    pub keysym: u32,
    pub modifiers: u32,
    pub action: Action,
}

pub const KEY_BINDINGS: &[KeyBinding] = &[
    KeyBinding {
        keysym: XK_Return,
        modifiers: MOD_KEY as u32,
        action: Action::OpenTerminal,
    },
    KeyBinding {
        keysym: XK_d,
        modifiers: MOD_KEY as u32,
        action: Action::OpenLauncher,
    },
    KeyBinding {
        keysym: XK_q,
        modifiers: MOD_KEY as u32,
        action: Action::QuitWM,
    },
    KeyBinding {
        keysym: XK_j,
        modifiers: MOD_KEY as u32,
        action: Action::FocusNext,
    },
    KeyBinding {
        keysym: XK_k,
        modifiers: MOD_KEY as u32,
        action: Action::FocusPrev,
    },
    KeyBinding {
        keysym: XK_o,
        modifiers: MOD_KEY as u32,
        action: Action::MonitorNext,
    },
    KeyBinding {
        keysym: XK_i,
        modifiers: MOD_KEY as u32,
        action: Action::MonitorPrev,
    },
    KeyBinding {
        keysym: XK_c,
        modifiers: MOD_KEY as u32,
        action: Action::CloseWindow,
    },
];
