use std::ffi::CString;
use std::mem::MaybeUninit;
use std::os::raw::{c_int, c_uchar};
use std::ptr::null;

use x11::xinput2::{
    XIAllDevices, XIDeviceEvent, XIEventMask, XISelectEvents, XISetMask, XI_KeyPress,
    XI_KeyRelease, XI_LASTEVENT,
};
use x11::xlib::{
    Display, False, GenericEvent, XDefaultRootWindow, XEvent, XFreeEventData, XGenericEventCookie,
    XGetEventData, XNextEvent, XOpenDisplay, XQueryExtension, XSync,
};

use cstr::cstr;
use libc::{calloc, size_t};

pub struct XInput {
    xi_opcode: c_int,
    display: *mut Display,
}

impl XInput {
    pub fn new(display: Option<&str>) -> XInput {
        let mut xi_opcode = 0 as c_int;
        unsafe {
            let mut event = 0 as c_int;
            let mut error = 0 as c_int;
            let display = match display {
                None => XOpenDisplay(null()),
                Some(d) => {
                    let d = CString::new(d).unwrap();
                    XOpenDisplay(d.as_ptr())
                }
            };
            if display.is_null() {
                panic!("Unable to connect to X server");
            }

            if XQueryExtension(
                display,
                cstr!("XInputExtension").as_ptr(),
                &mut xi_opcode as *mut c_int,
                &mut event as *mut c_int,
                &mut error as *mut c_int,
            ) == 0
            {
                panic!("X Input extension not available");
            }

            let window = XDefaultRootWindow(display);
            let mask = MaybeUninit::<XIEventMask>::uninit();
            let mut mask = mask.assume_init();
            mask.deviceid = XIAllDevices;
            mask.mask_len = (XI_LASTEVENT >> 3) + 1;
            mask.mask = calloc(mask.mask_len as size_t, 1) as *mut c_uchar;

            let m = std::slice::from_raw_parts_mut(mask.mask, mask.mask_len as usize);
            XISetMask(m, XI_KeyPress);
            XISetMask(m, XI_KeyRelease);

            XISelectEvents(display, window, &mut mask as *mut XIEventMask, 1);
            XSync(display, False as c_int);

            Self { xi_opcode, display }
        }
    }

    pub fn next_event(&self) -> Option<(XGenericEventCookie, XIDeviceEvent)> {
        let display = self.display;
        unsafe {
            let ev = MaybeUninit::<XEvent>::uninit();
            let mut ev = ev.assume_init();
            let cookie = &mut ev.generic_event_cookie as *mut XGenericEventCookie;
            XNextEvent(display, &mut ev as *mut XEvent);

            let mut result = None;

            if XGetEventData(display, cookie) != 0
                && (*cookie).type_ == GenericEvent
                && (*cookie).extension == self.xi_opcode
            {
                let cookie = &*cookie;
                let event = &*(cookie.data as *const XIDeviceEvent);

                result = Some((*cookie, *event));
            }

            XFreeEventData(display, cookie);

            result
        }
    }
}
