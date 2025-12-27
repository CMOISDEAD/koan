use x11::keysym::{XK_Return, XK_c, XK_comma, XK_d, XK_j, XK_k, XK_period, XK_q};
use x11::xlib::{Mod1Mask, ShiftMask};

#[derive(Debug, Clone, Copy)]
pub enum Action {
    OpenLauncher,
    OpenTerminal,
    QuitWM,
    FocusNext,
    FocusPrev,
    MonitorNext,
    MonitorPrev,
    MoveWindowToNextMonitor,
    MoveWindowToPrevMonitor,
    CloseWindow,
}

#[derive(Debug)]
pub struct KeyBinding {
    pub keysym: u32,
    pub modifiers: u32,
    pub action: Action,
}

const MOD_KEY: u32 = Mod1Mask;

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
        keysym: XK_comma,
        modifiers: MOD_KEY as u32,
        action: Action::MonitorPrev,
    },
    KeyBinding {
        keysym: XK_period,
        modifiers: MOD_KEY as u32,
        action: Action::MonitorNext,
    },
    KeyBinding {
        keysym: XK_comma,
        modifiers: (MOD_KEY | ShiftMask) as u32,
        action: Action::MoveWindowToPrevMonitor,
    },
    KeyBinding {
        keysym: XK_period,
        modifiers: (MOD_KEY | ShiftMask) as u32,
        action: Action::MoveWindowToNextMonitor,
    },
    KeyBinding {
        keysym: XK_c,
        modifiers: (MOD_KEY | ShiftMask) as u32,
        action: Action::CloseWindow,
    },
    KeyBinding {
        keysym: XK_q,
        modifiers: (MOD_KEY | ShiftMask) as u32,
        action: Action::QuitWM,
    },
];

pub const BORDER_WIDTH: u32 = 1;
