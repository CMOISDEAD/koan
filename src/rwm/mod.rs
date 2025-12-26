pub mod config;
use crate::rwm::config::{Action, KEY_BINDINGS};
use csscolorparser;
use std::collections::{BTreeSet, HashMap, HashSet}; // Añadido HashMap
use std::mem::zeroed;
use std::ptr::null;
use std::slice;
use thiserror::Error;
use x11::{xinerama, xlib};

const BORDER_WIDTH: u32 = 2;

#[derive(Error, Debug)]
pub enum MiniWMError {
    #[error("display not found")]
    DisplayNotFound,
    #[error("screen not found")]
    ScreenNotFound,
    #[error("generic error")]
    GenericError(String),
}

#[derive(Clone, Copy, Debug)]
pub struct Monitor {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub index: usize, // Añadido índice para identificarlo fácil
}

pub type Window = u64;

pub struct MiniWM {
    monitors: Vec<Monitor>,
    current_monitor: usize, // Índice del monitor donde está el mouse/foco
    display: *mut xlib::Display,

    windows: BTreeSet<Window>,               // Todas las ventanas activas
    window_monitors: HashMap<Window, usize>, // Mapa: Ventana -> Índice de Monitor

    focused: Option<Window>,

    // atoms
    wm_protocols: xlib::Atom,
    wm_delete: xlib::Atom,
}

impl MiniWM {
    pub fn new() -> Result<Self, MiniWMError> {
        unsafe {
            let display = xlib::XOpenDisplay(null());
            if display.is_null() {
                return Err(MiniWMError::DisplayNotFound);
            }
            let wm_protocols = xlib::XInternAtom(display, "WM_PROTOCOLS\0".as_ptr() as *const _, 0);
            let wm_delete =
                xlib::XInternAtom(display, "WM_DELETE_WINDOW\0".as_ptr() as *const _, 0);

            Ok(Self {
                display,
                windows: BTreeSet::new(),
                window_monitors: HashMap::new(),
                focused: None,
                monitors: Vec::new(),
                current_monitor: 0,
                wm_protocols,
                wm_delete,
            })
        }
    }

    pub fn init(&mut self) -> Result<(), MiniWMError> {
        unsafe {
            let root = xlib::XDefaultRootWindow(self.display);
            xlib::XSelectInput(
                self.display,
                root,
                xlib::SubstructureRedirectMask
                    | xlib::SubstructureNotifyMask
                    | xlib::FocusChangeMask
                    | xlib::EnterWindowMask
                    | xlib::StructureNotifyMask,
            );

            self.update_monitors();

            // Re-grab keys por si acaso
            xlib::XUngrabKey(self.display, xlib::AnyKey, xlib::AnyModifier, root);

            for binding in KEY_BINDINGS {
                let keycode = xlib::XKeysymToKeycode(self.display, binding.keysym as u64);
                let status = xlib::XGrabKey(
                    self.display,
                    keycode as i32,
                    binding.modifiers,
                    root,
                    0,
                    xlib::GrabModeAsync,
                    xlib::GrabModeAsync,
                );
                if status == 0 {
                    eprintln!("Warning: XGrabKey failed for keysym {}", binding.keysym);
                }
            }
        }
        Ok(())
    }

    pub fn run(&mut self) -> Result<(), MiniWMError> {
        let mut event: xlib::XEvent = unsafe { zeroed() };
        loop {
            unsafe {
                xlib::XNextEvent(self.display, &mut event);
                match event.get_type() {
                    xlib::ConfigureNotify => {
                        let ev = xlib::XConfigureEvent::from(event);
                        let root = xlib::XDefaultRootWindow(self.display);

                        if ev.window == root {
                            self.update_monitors();
                            self.layout()?;
                        }
                    }
                    xlib::MapRequest => {
                        let ev = xlib::XMapRequestEvent::from(event);
                        self.manage_window(ev.window)?;
                    }
                    xlib::UnmapNotify => {
                        let ev = xlib::XUnmapEvent::from(event);
                        self.handle_unmap(ev.window)?;
                    }
                    xlib::EnterNotify => {
                        let ev = xlib::XEnterWindowEvent::from(event);
                        self.handle_enter_notify(ev);
                    }
                    xlib::KeyPress => {
                        if let Err(_) = self.handle_keypress(xlib::XKeyEvent::from(event)) {
                            return Ok(());
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    // --- Manejo de Monitores ---

    fn update_monitors(&mut self) {
        unsafe {
            let mut num = 0i32;
            let screens_ptr = xinerama::XineramaQueryScreens(self.display, &mut num);

            self.monitors.clear();

            if screens_ptr.is_null() || num <= 0 {
                // Fallback un solo monitor
                let root = xlib::XDefaultRootWindow(self.display);
                let mut root_attr: xlib::XWindowAttributes = zeroed();
                xlib::XGetWindowAttributes(self.display, root, &mut root_attr);
                self.monitors.push(Monitor {
                    x: 0,
                    y: 0,
                    width: root_attr.width,
                    height: root_attr.height,
                    index: 0,
                });
            } else {
                let screens = slice::from_raw_parts(screens_ptr, num as usize);
                for (i, screen) in screens.iter().enumerate() {
                    self.monitors.push(Monitor {
                        x: screen.x_org as i32,
                        y: screen.y_org as i32,
                        width: screen.width as i32,
                        height: screen.height as i32,
                        index: i,
                    });
                }
                xlib::XFree(screens_ptr as *mut _);
            }

            // VALIDACIÓN POST-CAMBIO:
            // Si ahora hay menos monitores que antes, mueve las ventanas "huérfanas" al monitor 0
            let total_monitors = self.monitors.len();
            for mon_idx in self.window_monitors.values_mut() {
                if *mon_idx >= total_monitors {
                    *mon_idx = 0;
                }
            }

            if self.current_monitor >= total_monitors {
                self.current_monitor = 0;
            }
        }
    }

    // --- Lógica de Eventos ---

    fn handle_enter_notify(&mut self, ev: xlib::XEnterWindowEvent) {
        // 1. Determinar en qué monitor está el mouse
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

        // 2. Si entramos a una de nuestras ventanas gestionadas, darle foco
        if self.windows.contains(&ev.window) {
            self.focus_window(ev.window);
        }
        // Si es la root window, solo actualizamos el monitor, no enfocamos nada específico
    }

    fn handle_unmap(&mut self, window: Window) -> Result<(), MiniWMError> {
        let was_focused = self.focused == Some(window);

        // Determinar en qué monitor estaba para enfocar otra en ese mismo monitor
        let monitor_idx = self.window_monitors.remove(&window).unwrap_or(0);
        self.windows.remove(&window);

        if was_focused {
            self.focused = None;
            // Buscar la última ventana disponible EN EL MISMO MONITOR
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

    fn handle_keypress(&mut self, ke: xlib::XKeyEvent) -> Result<(), MiniWMError> {
        unsafe {
            let keysym = xlib::XKeycodeToKeysym(self.display, ke.keycode as u8, 0);
            for binding in KEY_BINDINGS {
                if keysym == binding.keysym as u64
                    && (ke.state & binding.modifiers) == binding.modifiers
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
                            if !self.monitors.is_empty() {
                                // Ciclo infinito hacia atrás: si es 0, va al último
                                self.current_monitor = if self.current_monitor == 0 {
                                    self.monitors.len() - 1
                                } else {
                                    self.current_monitor - 1
                                };
                                self.focus_monitor(self.current_monitor);
                            }
                        }
                        Action::MonitorNext => {
                            if !self.monitors.is_empty() {
                                // Ciclo infinito hacia adelante: (actual + 1) % total
                                self.current_monitor =
                                    (self.current_monitor + 1) % self.monitors.len();
                                self.focus_monitor(self.current_monitor);
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn focus_monitor(&mut self, index: usize) {
        let mon = self.monitors[index];

        // 1. Mover el mouse al centro del monitor seleccionado
        // Esto evita que el foco "rebote" por culpa del Focus follows mouse
        unsafe {
            xlib::XWarpPointer(
                self.display,
                0, // None
                xlib::XDefaultRootWindow(self.display),
                0,
                0,
                0,
                0,
                mon.x + (mon.width / 2),
                mon.y + (mon.height / 2),
            );
            xlib::XFlush(self.display);
        }

        // 2. Enfocar la última ventana activa en ese monitor (si existe)
        let last_win = self
            .windows
            .iter()
            .filter(|&w| self.window_monitors.get(w) == Some(&index))
            .last()
            .copied();

        if let Some(win) = last_win {
            self.focus_window(win);
        } else {
            // Si el monitor está vacío, quitamos el foco de la ventana actual
            self.focused = None;
        }
    }

    // Lógica unificada para ciclar ventanas DENTRO del monitor actual
    fn cycle_focus(&mut self, next: bool) {
        // Obtenemos las ventanas del monitor actual en un Vector ordenado
        let current_windows: Vec<Window> = self
            .windows
            .iter()
            .filter(|w| self.window_monitors.get(w) == Some(&self.current_monitor))
            .copied()
            .collect();

        if current_windows.is_empty() {
            return;
        }

        let target = if let Some(focused) = self.focused {
            // Buscar índice actual
            if let Some(pos) = current_windows.iter().position(|&w| w == focused) {
                if next {
                    // Siguiente (wrap around)
                    let next_idx = (pos + 1) % current_windows.len();
                    current_windows[next_idx]
                } else {
                    // Anterior (wrap around)
                    let prev_idx = if pos == 0 {
                        current_windows.len() - 1
                    } else {
                        pos - 1
                    };
                    current_windows[prev_idx]
                }
            } else {
                // Si la ventana enfocada no está en este monitor (caso raro), coger la primera
                current_windows[0]
            }
        } else {
            // Si no hay foco, coger la primera o la última
            if next {
                current_windows[0]
            } else {
                *current_windows.last().unwrap()
            }
        };

        self.focus_window(target);
    }

    fn manage_window(&mut self, win: Window) -> Result<(), MiniWMError> {
        // Añadir a la lista global
        self.windows.insert(win);

        // Asignar al monitor donde está el mouse actualmente (current_monitor)
        self.window_monitors.insert(win, self.current_monitor);

        unsafe {
            xlib::XSelectInput(
                self.display,
                win,
                xlib::EnterWindowMask | xlib::FocusChangeMask,
            );
            xlib::XSetWindowBorderWidth(self.display, win, BORDER_WIDTH);
        }

        self.layout()?;

        unsafe {
            xlib::XMapRaised(self.display, win);
            xlib::XFlush(self.display);
        }

        self.focus_window(win);
        Ok(())
    }

    fn focus_window(&mut self, window: Window) {
        // Si cambiamos de foco, despintar la anterior
        if let Some(old_win) = self.focused {
            if old_win != window {
                self.set_border_color(old_win, "#888888"); // Gris
            }
        }

        // Actualizar monitor actual basado en la ventana que enfocamos
        if let Some(&mon_idx) = self.window_monitors.get(&window) {
            self.current_monitor = mon_idx;
        }

        self.focused = Some(window);

        unsafe {
            xlib::XSetInputFocus(
                self.display,
                window,
                xlib::RevertToPointerRoot,
                xlib::CurrentTime,
            );
            // Raise window es opcional en tiling puro, pero útil aquí
            xlib::XRaiseWindow(self.display, window);
        }

        self.set_border_color(window, "#00ff00"); // Verde
    }

    fn set_border_color(&self, window: Window, color: &str) {
        let pixel = self.get_pixel_from_color(color);
        unsafe {
            xlib::XSetWindowBorder(self.display, window, pixel);
            xlib::XFlush(self.display);
        }
    }

    fn get_pixel_from_color(&self, color_str: &str) -> u64 {
        let css_color = match csscolorparser::parse(color_str) {
            Ok(c) => c,
            Err(_) => return 0, // Fallback negro silencioso
        };
        let [r, g, b, _] = css_color.to_rgba8();
        let red = (r as u16) * 257;
        let green = (g as u16) * 257;
        let blue = (b as u16) * 257;

        let mut xcolor: xlib::XColor = unsafe { zeroed() };
        xcolor.red = red;
        xcolor.green = green;
        xcolor.blue = blue;

        let colormap =
            unsafe { xlib::XDefaultColormap(self.display, xlib::XDefaultScreen(self.display)) };
        unsafe {
            if xlib::XAllocColor(self.display, colormap, &mut xcolor) == 0 {
                return 0;
            }
        }
        xcolor.pixel as u64
    }

    fn send_delete(&mut self, window: Window) {
        unsafe {
            let mut client_ev = xlib::XClientMessageEvent {
                type_: xlib::ClientMessage,
                serial: 0,
                send_event: 1,
                display: self.display,
                window,
                message_type: self.wm_protocols,
                format: 32,
                data: xlib::ClientMessageData::new(),
            };
            client_ev.data.set_long(0, self.wm_delete as i64);
            client_ev.data.set_long(1, xlib::CurrentTime as i64);
            let mut event: xlib::XEvent = client_ev.into();
            xlib::XSendEvent(self.display, window, 0, xlib::NoEventMask, &mut event);
            xlib::XFlush(self.display);
        }
    }

    // --- Layout Multi-Monitor ---

    fn layout(&mut self) -> Result<(), MiniWMError> {
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

                // CORRECCIÓN: Restar el doble del borde para que la ventana + borde = espacio total
                // Usamos saturating_sub para evitar que el valor sea negativo si la ventana es muy pequeña
                let final_w = (width as u32).saturating_sub(BORDER_WIDTH * 2);
                let final_h = (mon.height as u32).saturating_sub(BORDER_WIDTH * 2);

                self.move_window(window, current_x, mon.y);
                self.resize_window(window, final_w, final_h);

                current_x += width;
            }
        }
        Ok(())
    }

    fn move_window(&self, window: Window, x: i32, y: i32) {
        unsafe { xlib::XMoveWindow(self.display, window, x, y) };
    }

    fn resize_window(&self, window: Window, width: u32, height: u32) {
        let safe_width = if width < 1 { 1 } else { width };
        let safe_height = if height < 1 { 1 } else { height };
        unsafe { xlib::XResizeWindow(self.display, window, safe_width, safe_height) };
    }
}
