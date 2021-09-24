use core_graphics::event::CGKeyCode;

use crate::event::Key;

use crate::macos::native::keycode::*;

macro_rules! decl_keycodes {
    ($($key:ident, $code:literal),*) => {

        impl Key {
            pub fn to_raw(&self) -> Option<CGKeyCode> {
                match self {
                    $(
                        Key::$key => Some($code),
                    )*
                    _ => None,
                }
            }
    
            pub fn from_raw(code: CGKeyCode) -> Option<Key> {
                match code {
                    $(
                        $code => Some(Key::$key),
                    )*
                    _ => None,
                }
            }
        }
    }
}

decl_keycodes! {
    LeftAlt, ALT,
    RightAlt, ALT_GR,
    Backspace, BACKSPACE,
    CapsLock, CAPS_LOCK,
    Controlpanel, CONTROL_LEFT,
    Down, DOWN_ARROW,
    Esc, ESCAPE,
    F1, F1,
    F10, F10,
    F11, F11,
    F12, F12,
    F2, F2,
    F3, F3,
    F4, F4,
    F5, F5,
    F6, F6,
    F7, F7,
    F8, F8,
    F9, F9,

}