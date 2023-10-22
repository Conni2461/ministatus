use std::ffi::{c_ulong, CString};
use std::ptr::null;

pub struct Window {
    dpy: *mut x11::xlib::Display,
    window: c_ulong,
}

impl Window {
    pub fn new() -> Self {
        unsafe {
            let dpy = x11::xlib::XOpenDisplay(null());
            let screen = x11::xlib::XDefaultScreen(dpy);
            let window = x11::xlib::XRootWindow(dpy, screen);
            Self { dpy, window }
        }
    }

    pub fn set_title(&self, s: &str) -> Result<(), anyhow::Error> {
        let name = CString::new(s)?;
        unsafe {
            x11::xlib::XStoreName(self.dpy, self.window, name.as_ptr());
            x11::xlib::XFlush(self.dpy);
        }
        Ok(())
    }
}
