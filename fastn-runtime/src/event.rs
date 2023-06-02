#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Event {
    // FocusGained,
    // FocusLost,
    // Key { code: u32, pressed: bool },
    // Mouse { x: u32, y: u32, pressed: bool },
    // Resize(u16, u16),
    OnMouseEnter,
}

impl From<i32> for Event {
    fn from(i: i32) -> Event {
        match i {
            0 => Event::OnMouseEnter,
            _ => panic!("Unknown UIProperty: {}", i),
        }
    }
}

impl From<Event> for i32 {
    fn from(v: Event) -> i32 {
        match v {
            Event::OnMouseEnter => 0,
        }
    }
}
