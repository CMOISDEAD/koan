use csscolorparser;
use std::mem::zeroed;
use x11::xlib;

pub fn get_pixel_from_color(display: *mut xlib::Display, color_str: &str) -> u64 {
    let css_color = match csscolorparser::parse(color_str) {
        Ok(c) => c,
        Err(_) => return 0,
    };
    let [r, g, b, _] = css_color.to_rgba8();
    let red = (r as u16) * 257;
    let green = (g as u16) * 257;
    let blue = (b as u16) * 257;

    let mut xcolor: xlib::XColor = unsafe { zeroed() };
    xcolor.red = red;
    xcolor.green = green;
    xcolor.blue = blue;

    let colormap = unsafe { xlib::XDefaultColormap(display, xlib::XDefaultScreen(display)) };
    unsafe {
        if xlib::XAllocColor(display, colormap, &mut xcolor) == 0 {
            return 0;
        }
    }
    xcolor.pixel as u64
}
