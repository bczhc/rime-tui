use librime_sys::{
    RimeModifier_kAltMask, RimeModifier_kControlMask, RimeModifier_kReleaseMask,
    RimeModifier_kShiftMask, RimeModifier_kSuperMask,
};
use rime_api::KeyEvent;
use x11::keysym::*;
use x11::xinput2::XIDeviceEvent;
use x11::xlib;

pub struct KeyEventResolver<F>
where
    F: Fn(KeyEvent),
{
    shift_pressed: bool,
    ctrl_pressed: bool,
    alt_pressed: bool,
    super_pressed: bool,
    callback: F,
}

const KEY_CTRL_L: u32 = 37;
const KEY_CTRL_R: u32 = 105;
const KEY_SHIFT_L: u32 = 50;
const KEY_SHIFT_R: u32 = 62;
const KEY_ALT_L: u32 = 64;
const KEY_ALT_R: u32 = 108;
const KEY_SUPER: u32 = 133;

impl<F> KeyEventResolver<F>
where
    F: Fn(KeyEvent),
{
    #[allow(clippy::new_without_default)]
    pub fn new(callback: F) -> Self {
        Self {
            shift_pressed: false,
            ctrl_pressed: false,
            alt_pressed: false,
            super_pressed: false,
            callback,
        }
    }

    pub fn on_key_event(&mut self, event: &XIDeviceEvent) {
        // Note: can't get if a key gets consumed by Rime processors when using `send_key_sequence`
        // maybe there's a need to switch to `RimeProcessKey`
        let detail = event.detail as u32;

        let release = event.evtype == xlib::KeyRelease;

        let mut ke = KeyEvent::new(0, 0);

        // symbols will be changed when Shift pressed, e.g. XK_2 -> XK_at
        let code = match detail {
            d @ 24..=33 if self.shift_pressed => {
                [XK_Q, XK_W, XK_E, XK_R, XK_T, XK_Y, XK_U, XK_I, XK_O, XK_P][(d - 24) as usize]
            }
            d @ 38..=46 if self.shift_pressed => {
                [XK_A, XK_S, XK_D, XK_F, XK_G, XK_H, XK_J, XK_K, XK_L][(d - 38) as usize]
            }
            d @ 52..=58 if self.shift_pressed => {
                [XK_Z, XK_X, XK_C, XK_V, XK_B, XK_N, XK_M][(d - 52) as usize]
            }
            d @ 10..=19 if self.shift_pressed => [
                XK_exclam,
                XK_at,
                XK_numbersign,
                XK_dollar,
                XK_percent,
                XK_asciicircum,
                XK_ampersand,
                XK_asterisk,
                XK_parenleft,
                XK_parenright,
            ][(d - 10) as usize],
            d @ 24..=33 => {
                [XK_q, XK_w, XK_e, XK_r, XK_t, XK_y, XK_u, XK_i, XK_o, XK_p][(d - 24) as usize]
            }
            d @ 38..=46 => {
                [XK_a, XK_s, XK_d, XK_f, XK_g, XK_h, XK_j, XK_k, XK_l][(d - 38) as usize]
            }
            d @ 52..=58 => [XK_z, XK_x, XK_c, XK_v, XK_b, XK_n, XK_m][(d - 52) as usize],
            65 => XK_space,
            22 => XK_BackSpace,
            36 => XK_Return,
            9 => XK_Escape,
            20 => XK_minus,
            21 => XK_equal,
            34 => XK_bracketleft,
            35 => XK_bracketright,
            51 => XK_backslash,
            47 => XK_semicolon,
            48 => XK_apostrophe,
            59 => XK_comma,
            60 => XK_percent,
            61 => XK_slash,
            d @ 10..=19 => {
                [XK_1, XK_2, XK_3, XK_4, XK_5, XK_6, XK_7, XK_8, XK_9, XK_0][(d - 10) as usize]
            }
            111 => XK_Up,
            116 => XK_Down,
            113 => XK_Left,
            114 => XK_Right,
            23 => XK_Tab,
            KEY_ALT_L => XK_Alt_L,
            KEY_ALT_R => XK_Alt_R,
            KEY_SHIFT_L => XK_Shift_L,
            KEY_SHIFT_R => XK_Shift_R,
            KEY_CTRL_L => XK_Control_L,
            KEY_CTRL_R => XK_Control_R,
            _ => {
                0xffffff /* Void symbol */
            }
        } as i32;
        ke.key_code = code;
        if release {
            ke.modifiers |= RimeModifier_kReleaseMask as i32;
        }
        if self.shift_pressed {
            ke.modifiers |= RimeModifier_kShiftMask as i32;
        }
        if self.super_pressed {
            ke.modifiers |= RimeModifier_kSuperMask as i32;
        }
        if self.ctrl_pressed {
            ke.modifiers |= RimeModifier_kControlMask as i32;
        }
        if self.alt_pressed {
            ke.modifiers = RimeModifier_kAltMask as i32;
        }
        (self.callback)(ke);

        match event.evtype {
            xlib::KeyPress => match detail {
                KEY_CTRL_L | KEY_CTRL_R => self.ctrl_pressed = true,
                KEY_SHIFT_L | KEY_SHIFT_R => self.shift_pressed = true,
                KEY_ALT_L | KEY_ALT_R => self.alt_pressed = true,
                KEY_SUPER => self.super_pressed = true,
                _ => {}
            },
            xlib::KeyRelease => match detail {
                KEY_CTRL_L | KEY_CTRL_R => self.ctrl_pressed = false,
                KEY_SHIFT_L | KEY_SHIFT_R => self.shift_pressed = false,
                KEY_ALT_L | KEY_ALT_R => self.alt_pressed = false,
                KEY_SUPER => self.super_pressed = false,
                _ => {}
            },
            _ => {}
        };
    }
}

pub fn wrap(key: &str) -> String {
    format!("{{{}}}", key)
}
