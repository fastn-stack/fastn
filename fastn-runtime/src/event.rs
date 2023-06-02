#[derive(Debug, Clone, Copy)]
pub enum ExternalEvent {
    // FocusGained,
    // FocusLost,
    // Key { code: u32, pressed: bool },
    // Mouse { x: u32, y: u32, left: bool, right: bool },
    // Resize(u16, u16),
    CursorMoved { x: f64, y: f64 },
    NoOp,
}

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub enum DomEventKind {
    OnMouseEnter,
    OnMouseLeave,
}

impl From<i32> for DomEventKind {
    fn from(i: i32) -> DomEventKind {
        match i {
            0 => DomEventKind::OnMouseEnter,
            1 => DomEventKind::OnMouseLeave,
            _ => panic!("Unknown UIProperty: {}", i),
        }
    }
}

impl From<DomEventKind> for i32 {
    fn from(v: DomEventKind) -> i32 {
        match v {
            DomEventKind::OnMouseEnter => 0,
            DomEventKind::OnMouseLeave => 1,
        }
    }
}

impl ExternalEvent {
    pub fn is_nop(&self) -> bool {
        matches!(self, ExternalEvent::NoOp)
    }
}
