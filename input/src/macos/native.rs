use crate::Event;
use cocoa::{
    base::{id, nil},
    foundation::NSAutoreleasePool,
};
use core_graphics::event::{CGEvent, CGEventTapLocation, CGEventType};
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
    // println!("Event ref {:?}", cg_event_ptr);
    // let cg_event: CGEvent = transmute_copy::<*mut c_void, CGEvent>(&cg_event_ptr);
    // let opt = KEYBOARD_STATE.lock();
    // if let Ok(mut keyboard) = opt {
    //     if let Some(event) = convert(_type, &cg_event, &mut keyboard) {
    //         if let Some(callback) = &mut GLOBAL_CALLBACK {
    //             callback(event);
    //         }
    //     }
    // }
    // println!("Event ref END {:?}", cg_event_ptr);
    // cg_event_ptr
    cg_event
}

#[link(name = "Cocoa", kind = "framework")]
pub fn listen<T>(callback: T) -> Result<(), ListenError>
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
        if tap.is_null() {
            return Err(ListenError::EventTapError);
        }
        let _loop = CFMachPortCreateRunLoopSource(nil, tap, 0);
        if _loop.is_null() {
            return Err(ListenError::LoopSourceError);
        }

        let current_loop = CFRunLoopGetCurrent();
        CFRunLoopAddSource(current_loop, _loop, kCFRunLoopCommonModes);

        CGEventTapEnable(tap, true);
        CFRunLoopRun();
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

pub unsafe fn convert(
    _type: CGEventType,
    cg_event: &CGEvent,
) -> Option<Event> {
    let option_type = match _type {
        CGEventType::LeftMouseDown => Some(EventType::ButtonPress(Button::Left)),
        CGEventType::LeftMouseUp => Some(EventType::ButtonRelease(Button::Left)),
        CGEventType::RightMouseDown => Some(EventType::ButtonPress(Button::Right)),
        CGEventType::RightMouseUp => Some(EventType::ButtonRelease(Button::Right)),
        CGEventType::MouseMoved => {
            let point = cg_event.location();
            Some(EventType::MouseMove {
                x: point.x,
                y: point.y,
            })
        }
        CGEventType::KeyDown => {
            let code = cg_event.get_integer_value_field(EventField::KEYBOARD_EVENT_KEYCODE);
            Some(EventType::KeyPress(key_from_code(code.try_into().ok()?)))
        }
        CGEventType::KeyUp => {
            let code = cg_event.get_integer_value_field(EventField::KEYBOARD_EVENT_KEYCODE);
            Some(EventType::KeyRelease(key_from_code(code.try_into().ok()?)))
        }
        CGEventType::FlagsChanged => {
            let code = cg_event.get_integer_value_field(EventField::KEYBOARD_EVENT_KEYCODE);
            let code = code.try_into().ok()?;
            let flags = cg_event.get_flags();
            if flags < LAST_FLAGS {
                LAST_FLAGS = flags;
                Some(EventType::KeyRelease(key_from_code(code)))
            } else {
                LAST_FLAGS = flags;
                Some(EventType::KeyPress(key_from_code(code)))
            }
        }
        CGEventType::ScrollWheel => {
            let delta_y =
                cg_event.get_integer_value_field(EventField::SCROLL_WHEEL_EVENT_POINT_DELTA_AXIS_1);
            let delta_x =
                cg_event.get_integer_value_field(EventField::SCROLL_WHEEL_EVENT_POINT_DELTA_AXIS_2);
            Some(EventType::Wheel { delta_x, delta_y })
        }
        _ => None,
    };
    if let Some(event_type) = option_type {
        let name = match event_type {
            EventType::KeyPress(_) => {
                let code =
                    cg_event.get_integer_value_field(EventField::KEYBOARD_EVENT_KEYCODE) as u32;
                let flags = cg_event.get_flags();
                keyboard_state.create_string_for_key(code, flags)
            }
            _ => None,
        };
        return Some(Event {
            event_type,
            time: SystemTime::now(),
            name,
        });
    }
    None
}


mod keycode {
    use core_graphics::event::CGKeyCode;

    /// Option
    const ALT: CGKeyCode = 58;
    /// Option_Right
    const ALT_GR: CGKeyCode = 61;
    const BACKSPACE: CGKeyCode = 51;
    const CAPS_LOCK: CGKeyCode = 57;
    /// Control Right does not exist on Mac
    const CONTROL_LEFT: CGKeyCode = 59;
    const DOWN_ARROW: CGKeyCode = 125;
    const ESCAPE: CGKeyCode = 53;
    const F1: CGKeyCode = 122;
    const F10: CGKeyCode = 109;
    const F11: CGKeyCode = 103;
    const F12: CGKeyCode = 111;
    const F2: CGKeyCode = 120;
    const F3: CGKeyCode = 99;
    const F4: CGKeyCode = 118;
    const F5: CGKeyCode = 96;
    const F6: CGKeyCode = 97;
    const F7: CGKeyCode = 98;
    const F8: CGKeyCode = 100;
    const F9: CGKeyCode = 101;
    const FUNCTION: CGKeyCode = 63;
    const LEFT_ARROW: CGKeyCode = 123;
    const META_LEFT: CGKeyCode = 55;
    const META_RIGHT: CGKeyCode = 54;
    const RETURN: CGKeyCode = 36;
    const RIGHT_ARROW: CGKeyCode = 124;
    const SHIFT_LEFT: CGKeyCode = 56;
    const SHIFT_RIGHT: CGKeyCode = 60;
    const SPACE: CGKeyCode = 49;
    const TAB: CGKeyCode = 48;
    const UP_ARROW: CGKeyCode = 126;
    const BACK_QUOTE: CGKeyCode = 50;
    const NUM1: CGKeyCode = 18;
    const NUM2: CGKeyCode = 19;
    const NUM3: CGKeyCode = 20;
    const NUM4: CGKeyCode = 21;
    const NUM5: CGKeyCode = 23;
    const NUM6: CGKeyCode = 22;
    const NUM7: CGKeyCode = 26;
    const NUM8: CGKeyCode = 28;
    const NUM9: CGKeyCode = 25;
    const NUM0: CGKeyCode = 29;
    const MINUS: CGKeyCode = 27;
    const EQUAL: CGKeyCode = 24;
    const KEY_Q: CGKeyCode = 12;
    const KEY_W: CGKeyCode = 13;
    const KEY_E: CGKeyCode = 14;
    const KEY_R: CGKeyCode = 15;
    const KEY_T: CGKeyCode = 17;
    const KEY_Y: CGKeyCode = 16;
    const KEY_U: CGKeyCode = 32;
    const KEY_I: CGKeyCode = 34;
    const KEY_O: CGKeyCode = 31;
    const KEY_P: CGKeyCode = 35;
    const LEFT_BRACKET: CGKeyCode = 33;
    const RIGHT_BRACKET: CGKeyCode = 30;
    const KEY_A: CGKeyCode = 0;
    const KEY_S: CGKeyCode = 1;
    const KEY_D: CGKeyCode = 2;
    const KEY_F: CGKeyCode = 3;
    const KEY_G: CGKeyCode = 5;
    const KEY_H: CGKeyCode = 4;
    const KEY_J: CGKeyCode = 38;
    const KEY_K: CGKeyCode = 40;
    const KEY_L: CGKeyCode = 37;
    const SEMI_COLON: CGKeyCode = 41;
    const QUOTE: CGKeyCode = 39;
    const BACK_SLASH: CGKeyCode = 42;
    const KEY_Z: CGKeyCode = 6;
    const KEY_X: CGKeyCode = 7;
    const KEY_C: CGKeyCode = 8;
    const KEY_V: CGKeyCode = 9;
    const KEY_B: CGKeyCode = 11;
    const KEY_N: CGKeyCode = 45;
    const KEY_M: CGKeyCode = 46;
    const COMMA: CGKeyCode = 43;
    const DOT: CGKeyCode = 47;
    const SLASH: CGKeyCode = 44;
}
