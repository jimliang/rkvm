use crate::{Button, Direction, Event, Key, KeyKind};
use cocoa::{
    base::{id, nil},
    foundation::NSAutoreleasePool,
};
use core_graphics::event::{CGEvent, CGEventTapLocation, CGEventType, EventField};
use std::convert::TryInto;
use std::io::{Error, ErrorKind};
use std::os::raw::c_void;

pub type CFMachPortRef = *const c_void;
pub type CFIndex = u64;
pub type CFAllocatorRef = id;
pub type CFRunLoopSourceRef = id;
pub type CFRunLoopRef = id;
pub type CFRunLoopMode = id;
pub type CGEventTapProxy = id;
pub type CGEventRef = CGEvent;
pub type CGEventTapPlacement = u32;
pub const kCGHeadInsertEventTap: u32 = 0;
pub type CGEventMask = u64;
#[allow(non_upper_case_globals)]
pub const kCGEventMaskForAllEvents: u64 = (1 << CGEventType::LeftMouseDown as u64)
    + (1 << CGEventType::LeftMouseUp as u64)
    + (1 << CGEventType::RightMouseDown as u64)
    + (1 << CGEventType::RightMouseUp as u64)
    + (1 << CGEventType::MouseMoved as u64)
    + (1 << CGEventType::LeftMouseDragged as u64)
    + (1 << CGEventType::RightMouseDragged as u64)
    + (1 << CGEventType::KeyDown as u64)
    + (1 << CGEventType::KeyUp as u64)
    + (1 << CGEventType::FlagsChanged as u64)
    + (1 << CGEventType::ScrollWheel as u64);
#[repr(u32)]
pub enum CGEventTapOption {
    Default = 0,
    ListenOnly = 1,
}

static mut GLOBAL_CALLBACK: Option<Box<dyn FnMut(Event)>> = None;

unsafe extern "C" fn raw_callback(
    _proxy: CGEventTapProxy,
    _type: CGEventType,
    cg_event: CGEventRef,
    _user_info: *mut c_void,
) -> CGEventRef {
    // println!("raaw {:?}", _type);
    if let Some(event) = convert(_type, &cg_event) {
        if let Some(callback) = &mut GLOBAL_CALLBACK {
            callback(event);
        }
    }
    cg_event
}

#[link(name = "Cocoa", kind = "framework")]
pub fn listen<T>(callback: T) -> Result<(), Error>
where
    T: FnMut(Event) + 'static,
{
    unsafe {
        GLOBAL_CALLBACK = Some(Box::new(callback));
        let _pool = NSAutoreleasePool::new(nil);
        let tap = CGEventTapCreate(
            CGEventTapLocation::HID, // HID, Session, AnnotatedSession,
            kCGHeadInsertEventTap,
            CGEventTapOption::ListenOnly,
            kCGEventMaskForAllEvents,
            raw_callback,
            nil,
        );
        println!("tap {:?}", tap);
        if tap.is_null() {
            return Err(Error::new(ErrorKind::Other, "Create EventTap error"));
        }
        let _loop = CFMachPortCreateRunLoopSource(nil, tap, 0);
        if _loop.is_null() {
            return Err(Error::new(ErrorKind::Other, "Loop source error"));
        }

        let current_loop = CFRunLoopGetCurrent();
        CFRunLoopAddSource(current_loop, _loop, kCFRunLoopCommonModes);

        CGEventTapEnable(tap, true);
        CFRunLoopRun();
        println!("loop");
    }
    Ok(())
}

#[cfg(target_os = "macos")]
#[link(name = "Cocoa", kind = "framework")]
extern "C" {
    #[allow(improper_ctypes)]
    pub fn CGEventTapCreate(
        tap: CGEventTapLocation,
        place: CGEventTapPlacement,
        options: CGEventTapOption,
        eventsOfInterest: CGEventMask,
        callback: QCallback,
        user_info: id,
    ) -> CFMachPortRef;
    pub fn CFMachPortCreateRunLoopSource(
        allocator: CFAllocatorRef,
        tap: CFMachPortRef,
        order: CFIndex,
    ) -> CFRunLoopSourceRef;
    pub fn CFRunLoopAddSource(rl: CFRunLoopRef, source: CFRunLoopSourceRef, mode: CFRunLoopMode);
    pub fn CFRunLoopGetCurrent() -> CFRunLoopRef;
    pub fn CGEventTapEnable(tap: CFMachPortRef, enable: bool);
    pub fn CFRunLoopRun();

    pub static kCFRunLoopCommonModes: CFRunLoopMode;

}
pub type QCallback = unsafe extern "C" fn(
    proxy: CGEventTapProxy,
    _type: CGEventType,
    cg_event: CGEventRef,
    user_info: *mut c_void,
) -> CGEventRef;

pub unsafe fn convert(_type: CGEventType, cg_event: &CGEvent) -> Option<Event> {
    match _type {
        CGEventType::Null => return None,
        CGEventType::LeftMouseDown => Some(Event::Key {
            direction: Direction::Down,
            kind: KeyKind::Button(Button::Left),
        }),
        CGEventType::LeftMouseUp => Some(Event::Key {
            direction: Direction::Up,
            kind: KeyKind::Button(Button::Left),
        }),
        CGEventType::RightMouseDown => Some(Event::Key {
            direction: Direction::Down,
            kind: KeyKind::Button(Button::Right),
        }),
        CGEventType::RightMouseUp => Some(Event::Key {
            direction: Direction::Up,
            kind: KeyKind::Button(Button::Right),
        }),
        CGEventType::MouseMoved => {
            // Event::MouseMove
            None
        }
        // CGEventType::LeftMouseDragged => todo!(),
        // CGEventType::RightMouseDragged => todo!(),
        CGEventType::KeyDown => {
            let code = cg_event.get_integer_value_field(EventField::KEYBOARD_EVENT_KEYCODE);
            if let Some(key) = Key::from_raw(code.try_into().ok()?) {
                Some(Event::Key {
                    direction: Direction::Down,
                    kind: KeyKind::Key(key),
                })
            } else {
                None
            }
        }
        // CGEventType::KeyUp => todo!(),
        // CGEventType::FlagsChanged => todo!(),
        // CGEventType::ScrollWheel => todo!(),
        // CGEventType::TabletPointer => todo!(),
        // CGEventType::TabletProximity => todo!(),
        // CGEventType::OtherMouseDown => todo!(),
        // CGEventType::OtherMouseUp => todo!(),
        // CGEventType::OtherMouseDragged => todo!(),
        // CGEventType::TapDisabledByTimeout => todo!(),
        // CGEventType::TapDisabledByUserInput => todo!(),
        _ => None,
    }
    // let option_type = match _type {
    //     CGEventType::LeftMouseDown => Some(EventType::ButtonPress(Button::Left)),
    //     CGEventType::LeftMouseUp => Some(EventType::ButtonRelease(Button::Left)),
    //     CGEventType::RightMouseDown => Some(EventType::ButtonPress(Button::Right)),
    //     CGEventType::RightMouseUp => Some(EventType::ButtonRelease(Button::Right)),
    //     CGEventType::MouseMoved => {
    //         let point = cg_event.location();
    //         Some(EventType::MouseMove {
    //             x: point.x,
    //             y: point.y,
    //         })
    //     }
    //     CGEventType::KeyDown => {
    //         let code = cg_event.get_integer_value_field(EventField::KEYBOARD_EVENT_KEYCODE);
    //         Some(EventType::KeyPress(key_from_code(code.try_into().ok()?)))
    //     }
    //     CGEventType::KeyUp => {
    //         let code = cg_event.get_integer_value_field(EventField::KEYBOARD_EVENT_KEYCODE);
    //         Some(EventType::KeyRelease(key_from_code(code.try_into().ok()?)))
    //     }
    //     CGEventType::FlagsChanged => {
    //         let code = cg_event.get_integer_value_field(EventField::KEYBOARD_EVENT_KEYCODE);
    //         let code = code.try_into().ok()?;
    //         let flags = cg_event.get_flags();
    //         if flags < LAST_FLAGS {
    //             LAST_FLAGS = flags;
    //             Some(EventType::KeyRelease(key_from_code(code)))
    //         } else {
    //             LAST_FLAGS = flags;
    //             Some(EventType::KeyPress(key_from_code(code)))
    //         }
    //     }
    //     CGEventType::ScrollWheel => {
    //         let delta_y =
    //             cg_event.get_integer_value_field(EventField::SCROLL_WHEEL_EVENT_POINT_DELTA_AXIS_1);
    //         let delta_x =
    //             cg_event.get_integer_value_field(EventField::SCROLL_WHEEL_EVENT_POINT_DELTA_AXIS_2);
    //         Some(EventType::Wheel { delta_x, delta_y })
    //     }
    //     _ => None,
    // };
    // if let Some(event_type) = option_type {
    //     let name = match event_type {
    //         EventType::KeyPress(_) => {
    //             let code =
    //                 cg_event.get_integer_value_field(EventField::KEYBOARD_EVENT_KEYCODE) as u32;
    //             let flags = cg_event.get_flags();
    //             keyboard_state.create_string_for_key(code, flags)
    //         }
    //         _ => None,
    //     };
    //     return Some(Event {
    //         event_type,
    //         time: SystemTime::now(),
    //         name,
    //     });
    // }
}

pub mod keycode {
    use core_graphics::event::CGKeyCode;

    /// Option
    pub const ALT: CGKeyCode = 58;
    /// Option_Right
    pub const ALT_GR: CGKeyCode = 61;
    pub const BACKSPACE: CGKeyCode = 51;
    pub const CAPS_LOCK: CGKeyCode = 57;
    /// Control Right does not exist on Mac
    pub const CONTROL_LEFT: CGKeyCode = 59;
    pub const DOWN_ARROW: CGKeyCode = 125;
    pub const ESCAPE: CGKeyCode = 53;
    pub const F1: CGKeyCode = 122;
    pub const F10: CGKeyCode = 109;
    pub const F11: CGKeyCode = 103;
    pub const F12: CGKeyCode = 111;
    pub const F2: CGKeyCode = 120;
    pub const F3: CGKeyCode = 99;
    pub const F4: CGKeyCode = 118;
    pub const F5: CGKeyCode = 96;
    pub const F6: CGKeyCode = 97;
    pub const F7: CGKeyCode = 98;
    pub const F8: CGKeyCode = 100;
    pub const F9: CGKeyCode = 101;
    pub const FUNCTION: CGKeyCode = 63;
    pub const LEFT_ARROW: CGKeyCode = 123;
    pub const META_LEFT: CGKeyCode = 55;
    pub const META_RIGHT: CGKeyCode = 54;
    pub const RETURN: CGKeyCode = 36;
    pub const RIGHT_ARROW: CGKeyCode = 124;
    pub const SHIFT_LEFT: CGKeyCode = 56;
    pub const SHIFT_RIGHT: CGKeyCode = 60;
    pub const SPACE: CGKeyCode = 49;
    pub const TAB: CGKeyCode = 48;
    pub const UP_ARROW: CGKeyCode = 126;
    pub const BACK_QUOTE: CGKeyCode = 50;
    pub const NUM1: CGKeyCode = 18;
    pub const NUM2: CGKeyCode = 19;
    pub const NUM3: CGKeyCode = 20;
    pub const NUM4: CGKeyCode = 21;
    pub const NUM5: CGKeyCode = 23;
    pub const NUM6: CGKeyCode = 22;
    pub const NUM7: CGKeyCode = 26;
    pub const NUM8: CGKeyCode = 28;
    pub const NUM9: CGKeyCode = 25;
    pub const NUM0: CGKeyCode = 29;
    pub const MINUS: CGKeyCode = 27;
    pub const EQUAL: CGKeyCode = 24;
    pub const KEY_Q: CGKeyCode = 12;
    pub const KEY_W: CGKeyCode = 13;
    pub const KEY_E: CGKeyCode = 14;
    pub const KEY_R: CGKeyCode = 15;
    pub const KEY_T: CGKeyCode = 17;
    pub const KEY_Y: CGKeyCode = 16;
    pub const KEY_U: CGKeyCode = 32;
    pub const KEY_I: CGKeyCode = 34;
    pub const KEY_O: CGKeyCode = 31;
    pub const KEY_P: CGKeyCode = 35;
    pub const LEFT_BRACKET: CGKeyCode = 33;
    pub const RIGHT_BRACKET: CGKeyCode = 30;
    pub const KEY_A: CGKeyCode = 0;
    pub const KEY_S: CGKeyCode = 1;
    pub const KEY_D: CGKeyCode = 2;
    pub const KEY_F: CGKeyCode = 3;
    pub const KEY_G: CGKeyCode = 5;
    pub const KEY_H: CGKeyCode = 4;
    pub const KEY_J: CGKeyCode = 38;
    pub const KEY_K: CGKeyCode = 40;
    pub const KEY_L: CGKeyCode = 37;
    pub const SEMI_COLON: CGKeyCode = 41;
    pub const QUOTE: CGKeyCode = 39;
    pub const BACK_SLASH: CGKeyCode = 42;
    pub const KEY_Z: CGKeyCode = 6;
    pub const KEY_X: CGKeyCode = 7;
    pub const KEY_C: CGKeyCode = 8;
    pub const KEY_V: CGKeyCode = 9;
    pub const KEY_B: CGKeyCode = 11;
    pub const KEY_N: CGKeyCode = 45;
    pub const KEY_M: CGKeyCode = 46;
    pub const COMMA: CGKeyCode = 43;
    pub const DOT: CGKeyCode = 47;
    pub const SLASH: CGKeyCode = 44;
}
