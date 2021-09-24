mod key;
use crate::event::{Axis, Button, Direction, Event, KeyKind};
use cocoa::appkit::CGPoint;
use core_graphics::{
    event::{CGEvent, CGEventType, CGMouseButton, ScrollEventUnit},
    event_source::{CGEventSource, CGEventSourceStateID},
};

impl Event {
    pub(crate) fn to_raw(&self) -> Option<CGEvent> {
        let source = CGEventSource::new(CGEventSourceStateID::HIDSystemState).ok()?;
        match self {
            Event::MouseScroll { delta } => {
                let wheel_count = 2;
                CGEvent::new_scroll_event(
                    source,
                    ScrollEventUnit::PIXEL,
                    wheel_count,
                    *delta,
                    *delta,
                    0,
                )
                .ok()
            }
            Event::MouseMove { axis, delta } => {
              let point = unsafe { get_current_mouse_location()? };
              let delta = *delta as f64;
              let (x, y) = match axis {
                &Axis::X => (point.x + delta, point.y),
                &Axis::Y => (point.x, point.y + delta),
              };
              let target_point = CGPoint {
                x, y
              };
              CGEvent::new_mouse_event(source, CGEventType::MouseMoved, target_point, CGMouseButton::Left).ok()
            },
            Event::Key { direction, kind } => match kind {
                &KeyKind::Button(button) => {
                    let event = match (button, direction) {
                        (Button::Right, Direction::Up) => CGEventType::RightMouseUp,
                        (Button::Right, Direction::Down) => CGEventType::RightMouseDown,
                        (Button::Left, Direction::Up) => CGEventType::LeftMouseUp,
                        (Button::Left, Direction::Down) => CGEventType::LeftMouseDown,
                        _ => return None,
                    };
                    let point = unsafe { get_current_mouse_location()? };
                    CGEvent::new_mouse_event(source, event, point, CGMouseButton::Left).ok()
                }
                &KeyKind::Key(key) => {
                    let keydown = match direction {
                        Direction::Up => false,
                        Direction::Down => true,
                    };
                    return None;
                    // let keycode = key.to_raw()?;
                    // CGEvent::new_keyboard_event(source, keycode, keydown).ok()
                }
            },
        }
    }
}

unsafe fn get_current_mouse_location() -> Option<CGPoint> {
    let source = CGEventSource::new(CGEventSourceStateID::HIDSystemState).ok()?;
    let event = CGEvent::new(source).ok()?;
    Some(event.location())
}
