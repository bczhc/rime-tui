use x11::xinput2::XIDeviceEvent;
use x11::xlib;

pub struct KeyEventResolver<F>
where
    F: Fn(&str),
{
    shift_pressed: bool,
    ctrl_pressed: bool,
    alt_pressed: bool,
    callback: F,
}

const CTRL_L: u32 = 37;
const CTRL_R: u32 = 105;
const SHIFT_L: u32 = 50;
const SHIFT_R: u32 = 62;
const ALT_L: u32 = 64;
const ALT_R: u32 = 108;

impl<F> KeyEventResolver<F>
where
    F: Fn(&str),
{
    #[allow(clippy::new_without_default)]
    pub fn new(callback: F) -> Self {
        Self {
            shift_pressed: false,
            ctrl_pressed: false,
            alt_pressed: false,
            callback,
        }
    }

    pub fn on_key_event(&mut self, event: &XIDeviceEvent) {
        let detail = event.detail as u32;

        let release = event.evtype == xlib::KeyRelease;

        match detail {
            24..=33 => {
                let mut c = char::from("qwertyuiop".as_bytes()[(detail - 24) as usize]).to_string();
                if release {
                    c = format!("Release+{}", c);
                }
                (self.callback)(&c);
            }
            38..=46 => {
                let mut c = char::from("asdfghjkl".as_bytes()[(detail - 38) as usize]).to_string();
                if release {
                    c = format!("Release+{}", c);
                }
                (self.callback)(&c);
            }
            52..=58 => {
                let mut c = char::from("zxcvbnm".as_bytes()[(detail - 52) as usize]).to_string();
                if release {
                    c = format!("Release+{}", c);
                }
                (self.callback)(&c);
            }
            _ => {}
        }

        match event.evtype {
            xlib::KeyPress => match detail {
                CTRL_L | CTRL_R => self.ctrl_pressed = true,
                SHIFT_L | SHIFT_R => self.shift_pressed = true,
                ALT_L | ALT_R => self.alt_pressed = true,
                _ => {}
            },
            xlib::KeyRelease => match detail {
                CTRL_L | CTRL_R => self.ctrl_pressed = false,
                SHIFT_L | SHIFT_R => self.shift_pressed = false,
                ALT_L | ALT_R => self.alt_pressed = false,
                _ => {}
            },
            _ => {}
        };
    }
}
