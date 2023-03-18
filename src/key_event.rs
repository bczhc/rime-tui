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

        let compose = |name: &str| {
            if release {
                format!("{{Release+{}}}", name)
            } else {
                wrap(name)
            }
        };

        match detail {
            24..=33 => {
                let c = char::from("qwertyuiop".as_bytes()[(detail - 24) as usize]).to_string();
                (self.callback)(compose(&c).as_str());
            }
            38..=46 => {
                let c = char::from("asdfghjkl".as_bytes()[(detail - 38) as usize]).to_string();
                (self.callback)(compose(&c).as_str());
            }
            52..=58 => {
                let c = char::from("zxcvbnm".as_bytes()[(detail - 52) as usize]).to_string();
                (self.callback)(compose(&c).as_str());
            }
            65 => (self.callback)(compose("space").as_str()),
            22 => (self.callback)(compose("BackSpace").as_str()),
            36 => (self.callback)(compose("Return").as_str()),
            9 => (self.callback)(compose("Escape").as_str()),
            20 => (self.callback)(compose("minus").as_str()),
            21 => (self.callback)(compose("equal").as_str()),
            34 => (self.callback)(compose("bracketleft").as_str()),
            35 => (self.callback)(compose("bracketright").as_str()),
            51 => (self.callback)(compose("backslash").as_str()),
            47 => (self.callback)(compose("semicolon").as_str()),
            48 => (self.callback)(compose("apostrophe").as_str()),
            59 => (self.callback)(compose("comma").as_str()),
            60 => (self.callback)(compose("period").as_str()),
            61 => (self.callback)(compose("slash").as_str()),
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

pub fn wrap(key: &str) -> String {
    format!("{{{}}}", key)
}
