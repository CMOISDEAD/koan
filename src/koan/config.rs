use super::modeline::{Module, ModuleType};
use x11::keysym::{XK_Return, XK_Tab, XK_c, XK_comma, XK_d, XK_j, XK_k, XK_period, XK_q};
use x11::xlib::{ControlMask, Mod1Mask, ShiftMask};

#[allow(dead_code)]
pub enum Color {
    Primary,
    Secondary,
    Error,
}

impl Color {
    pub fn hex(self) -> &'static str {
        match self {
            Color::Primary => "#00ff00",
            Color::Secondary => "#888888",
            Color::Error => "#ff5555",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Action {
    SwitchLayout,
    OpenLauncher,
    OpenTerminal,
    QuitWM,
    FocusNext,
    FocusPrev,
    MonitorNext,
    MonitorPrev,
    SwapWindowNext,
    SwapWindowPrev,
    IcreaseSize,
    DecreaseSize,
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
        keysym: XK_Tab,
        modifiers: (MOD_KEY | ControlMask) as u32,
        action: Action::SwitchLayout,
    },
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
        keysym: XK_j,
        modifiers: (MOD_KEY | ShiftMask) as u32,
        action: Action::SwapWindowNext,
    },
    KeyBinding {
        keysym: XK_k,
        modifiers: (MOD_KEY | ShiftMask) as u32,
        action: Action::SwapWindowPrev,
    },
    KeyBinding {
        keysym: XK_j,
        modifiers: (MOD_KEY | ControlMask) as u32,
        action: Action::IcreaseSize,
    },
    KeyBinding {
        keysym: XK_k,
        modifiers: (MOD_KEY | ControlMask) as u32,
        action: Action::DecreaseSize,
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

pub const MODELINE_UPDATE_TIME: u64 = 5;
pub const MODELINE_HEIGHT: u32 = 10;
pub const BORDER_WIDTH: u32 = 1;
pub const GAPS: u32 = 4;

pub struct AppCommand {
    pub program: &'static str,
    pub args: &'static [&'static str],
}

pub const TERMINAL: AppCommand = AppCommand {
    program: "alacritty",
    args: &[],
};

pub const LAUNCHER: AppCommand = AppCommand {
    program: "rofi",
    args: &["-show", "drun"],
};

pub const AUTO_START: &[AppCommand] = &[
    AppCommand {
        program: "feh",
        args: &["--bg-max", "/home/doom/Pictures/Wallpapers/Gnu_Koan.png"],
    },
    AppCommand {
        program: "sh",
        args: &["/home/doom/.screenlayout/dual.sh"],
    },
    // AppCommand {
    //     program: "emacs",
    //     args: &["--daemon"],
    // },
];

pub static MODULES: &[Module] = &[
    Module {
        kind: ModuleType::WindowTitle
    },
    Module {
        kind: ModuleType::Clock,
    },
];
