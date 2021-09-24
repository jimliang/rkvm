use core_graphics::event::CGKeyCode;

use crate::event::Key;

impl Key {
    pub(crate) fn to_raw(&self) -> Option<CGKeyCode> {
        todo!()
    }

    pub(crate) fn from_raw(code: u16) -> Option<Self> {
        todo!()
    }
}
