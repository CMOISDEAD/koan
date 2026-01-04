use x11::keysym::{
    XK_Return, XK_Tab, XK_c, XK_comma, XK_d, XK_h, XK_j, XK_k, XK_l, XK_period, XK_q, XK_space,
};
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
            Color::Secondary => "#000000",
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
    FocusUp,
    FocusDown,
    FocusRight,
    FocusLeft,
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
    MonitorNext,
    MonitorPrev,
    IncreaseHeight,
    DecreaseHeight,
    IncreaseWidth,
    DecreaseWidth,
    MoveWindowToNextMonitor,
    MoveWindowToPrevMonitor,
    ToggleFloat,
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
    // Focus - MOD + hjkl (navegación)
    KeyBinding {
        keysym: XK_k,
        modifiers: MOD_KEY as u32,
        action: Action::FocusUp,
    },
    KeyBinding {
        keysym: XK_j,
        modifiers: MOD_KEY as u32,
        action: Action::FocusDown,
    },
    KeyBinding {
        keysym: XK_l,
        modifiers: MOD_KEY as u32,
        action: Action::FocusRight,
    },
    KeyBinding {
        keysym: XK_h,
        modifiers: MOD_KEY as u32,
        action: Action::FocusLeft,
    },
    // Move/Swap - MOD + Shift + hjkl (mover float o swap en layout)
    KeyBinding {
        keysym: XK_k,
        modifiers: (MOD_KEY | ShiftMask) as u32,
        action: Action::MoveUp,
    },
    KeyBinding {
        keysym: XK_j,
        modifiers: (MOD_KEY | ShiftMask) as u32,
        action: Action::MoveDown,
    },
    KeyBinding {
        keysym: XK_l,
        modifiers: (MOD_KEY | ShiftMask) as u32,
        action: Action::MoveRight,
    },
    KeyBinding {
        keysym: XK_h,
        modifiers: (MOD_KEY | ShiftMask) as u32,
        action: Action::MoveLeft,
    },
    // Resize - MOD + Control + hjkl
    KeyBinding {
        keysym: XK_j,
        modifiers: (MOD_KEY | ControlMask) as u32,
        action: Action::IncreaseHeight,
    },
    KeyBinding {
        keysym: XK_k,
        modifiers: (MOD_KEY | ControlMask) as u32,
        action: Action::DecreaseHeight,
    },
    KeyBinding {
        keysym: XK_h,
        modifiers: (MOD_KEY | ControlMask) as u32,
        action: Action::DecreaseWidth,
    },
    KeyBinding {
        keysym: XK_l,
        modifiers: (MOD_KEY | ControlMask) as u32,
        action: Action::IncreaseWidth,
    },
    // Monitor navigation - MOD + comma/period
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
    // Move window to monitor - MOD + Shift + comma/period
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
        keysym: XK_space,
        modifiers: MOD_KEY as u32,
        action: Action::ToggleFloat,
    },
    KeyBinding {
        keysym: XK_q,
        modifiers: (MOD_KEY | ShiftMask) as u32,
        action: Action::QuitWM,
    },
];

pub const CURSOR_ENTER_FOCUS: bool = false;
pub const MODELINE_UPDATE_TIME: u64 = 5;
pub const MODELINE_HEIGHT: u32 = 15;
pub const BORDER_WIDTH: u32 = 1;
pub const GAPS: u32 = 4;

pub struct AppCommand {
    pub program: &'static str,
    pub args: &'static [&'static str],
}

pub const TERMINAL: AppCommand = AppCommand {
    program: "xterm",
    args: &[],
};

pub const LAUNCHER: AppCommand = AppCommand {
    program: "dmenu_run",
    args: &[],
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
