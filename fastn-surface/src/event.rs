pub enum Event {
    FocusGained,
    FocusLost,
    Key { code: u32, pressed: bool },
    Mouse { x: u32, y: u32, pressed: bool },
    Resize(u16, u16),
}
