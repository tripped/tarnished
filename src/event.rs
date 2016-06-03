use sdl2::event::Event;
use sdl2::keyboard::Keycode;

/// A restricted subset of SDL events that we're particularly interested in.
#[derive(Clone)]
pub enum IOEvent {
    KeyDown(Keycode),
    KeyUp(Keycode),
    MouseMoved { x: i32, y: i32 },
    MouseDown { x: i32, y: i32 },
    MouseUp { x: i32, y: i32 },
    MouseWheel(i32),
}

unsafe impl Send for IOEvent {}
unsafe impl Sync for IOEvent {}

pub fn translate_event(e: Event) -> Option<IOEvent> {
    match e {
        Event::MouseMotion {x, y, ..} => Some(IOEvent::MouseMoved{x:x, y:y}),
        Event::MouseButtonDown {x, y, ..} => Some(IOEvent::MouseDown{x:x, y:y}),
        Event::MouseButtonUp {x, y, ..} => Some(IOEvent::MouseUp{x:x, y:y}),
        Event::MouseWheel {y, ..} => Some(IOEvent::MouseWheel(y)),
        Event::KeyDown {keycode: Some(k), ..} => Some(IOEvent::KeyDown(k)),
        Event::KeyUp {keycode: Some(k), ..} => Some(IOEvent::KeyUp(k)),
        _ => None,
    }
}
