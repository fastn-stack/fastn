#[derive(Debug, Clone, Copy)]
pub enum ExternalEvent {
    // FocusGained,
    // FocusLost,
    Key { code: VirtualKeyCode, pressed: bool },
    ModifierChanged(ModifiersState),
    // Mouse { x: u32, y: u32, left: bool, right: bool },
    // Resize(u16, u16),
    CursorMoved { x: f64, y: f64 },
    Focused(bool),
    NoOp,
}

#[derive(Default)]
pub struct MouseState {
    pub(crate) x: f64,
    pub(crate) y: f64,
    pub(crate) left_down: bool,
    pub(crate) right_down: bool,
}

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub enum DomEventKind {
    OnMouseEnter,
    OnMouseLeave,
    OnGlobalKey, /*(Vec<VirtualKeyCode>)*/
}

impl DomEventKind {
    pub(crate) fn is_key(&self) -> bool {
        matches!(self, DomEventKind::OnGlobalKey)
    }
}

impl From<i32> for DomEventKind {
    fn from(i: i32) -> DomEventKind {
        match i {
            0 => DomEventKind::OnMouseEnter,
            1 => DomEventKind::OnMouseLeave,
            2 => DomEventKind::OnGlobalKey,
            _ => panic!("Unknown UIProperty: {}", i),
        }
    }
}

impl From<DomEventKind> for i32 {
    fn from(v: DomEventKind) -> i32 {
        match v {
            DomEventKind::OnMouseEnter => 0,
            DomEventKind::OnMouseLeave => 1,
            DomEventKind::OnGlobalKey => 2,
        }
    }
}

impl ExternalEvent {
    pub fn is_nop(&self) -> bool {
        matches!(self, ExternalEvent::NoOp)
    }
}

// source:
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
#[repr(u32)]
pub enum VirtualKeyCode {
    /// The '1' key over the letters.
    Key1,
    /// The '2' key over the letters.
    Key2,
    /// The '3' key over the letters.
    Key3,
    /// The '4' key over the letters.
    Key4,
    /// The '5' key over the letters.
    Key5,
    /// The '6' key over the letters.
    Key6,
    /// The '7' key over the letters.
    Key7,
    /// The '8' key over the letters.
    Key8,
    /// The '9' key over the letters.
    Key9,
    /// The '0' key over the 'O' and 'P' keys.
    Key0,

    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,

    /// The Escape key, next to F1.
    Escape,

    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    F13,
    F14,
    F15,
    F16,
    F17,
    F18,
    F19,
    F20,
    F21,
    F22,
    F23,
    F24,

    /// Print Screen/SysRq.
    Snapshot,
    /// Scroll Lock.
    Scroll,
    /// Pause/Break key, next to Scroll lock.
    Pause,

    /// `Insert`, next to Backspace.
    Insert,
    Home,
    Delete,
    End,
    PageDown,
    PageUp,

    Left,
    Up,
    Right,
    Down,

    /// The Backspace key, right over Enter.
    // TODO: rename
    Back,
    /// The Enter key.
    Return,
    /// The space bar.
    Space,

    /// The "Compose" key on Linux.
    Compose,

    Caret,

    Numlock,
    Numpad0,
    Numpad1,
    Numpad2,
    Numpad3,
    Numpad4,
    Numpad5,
    Numpad6,
    Numpad7,
    Numpad8,
    Numpad9,
    NumpadAdd,
    NumpadDivide,
    NumpadDecimal,
    NumpadComma,
    NumpadEnter,
    NumpadEquals,
    NumpadMultiply,
    NumpadSubtract,

    AbntC1,
    AbntC2,
    Apostrophe,
    Apps,
    Asterisk,
    At,
    Ax,
    Backslash,
    Calculator,
    Capital,
    Colon,
    Comma,
    Convert,
    Equals,
    Grave,
    Kana,
    Kanji,
    LAlt,
    LBracket,
    LControl,
    LShift,
    LWin,
    Mail,
    MediaSelect,
    MediaStop,
    Minus,
    Mute,
    MyComputer,
    // also called "Next"
    NavigateForward,
    // also called "Prior"
    NavigateBackward,
    NextTrack,
    NoConvert,
    OEM102,
    Period,
    PlayPause,
    Plus,
    Power,
    PrevTrack,
    RAlt,
    RBracket,
    RControl,
    RShift,
    RWin,
    Semicolon,
    Slash,
    Sleep,
    Stop,
    Sysrq,
    Tab,
    Underline,
    Unlabeled,
    VolumeDown,
    VolumeUp,
    Wake,
    WebBack,
    WebFavorites,
    WebForward,
    WebHome,
    WebRefresh,
    WebSearch,
    WebStop,
    Yen,
    Copy,
    Paste,
    Cut,
}

fn char_to_virtual_key_code(c: char) -> Option<VirtualKeyCode> {
    // We only translate keys that are affected by keyboard layout.
    //
    // Note that since keys are translated in a somewhat "dumb" way (reading character)
    // there is a concern that some combination, i.e. Cmd+char, causes the wrong
    // letter to be received, and so we receive the wrong key.
    //
    // Implementation reference: https://github.com/WebKit/webkit/blob/82bae82cf0f329dbe21059ef0986c4e92fea4ba6/Source/WebCore/platform/cocoa/KeyEventCocoa.mm#L626
    Some(match c {
        'a' | 'A' => VirtualKeyCode::A,
        'b' | 'B' => VirtualKeyCode::B,
        'c' | 'C' => VirtualKeyCode::C,
        'd' | 'D' => VirtualKeyCode::D,
        'e' | 'E' => VirtualKeyCode::E,
        'f' | 'F' => VirtualKeyCode::F,
        'g' | 'G' => VirtualKeyCode::G,
        'h' | 'H' => VirtualKeyCode::H,
        'i' | 'I' => VirtualKeyCode::I,
        'j' | 'J' => VirtualKeyCode::J,
        'k' | 'K' => VirtualKeyCode::K,
        'l' | 'L' => VirtualKeyCode::L,
        'm' | 'M' => VirtualKeyCode::M,
        'n' | 'N' => VirtualKeyCode::N,
        'o' | 'O' => VirtualKeyCode::O,
        'p' | 'P' => VirtualKeyCode::P,
        'q' | 'Q' => VirtualKeyCode::Q,
        'r' | 'R' => VirtualKeyCode::R,
        's' | 'S' => VirtualKeyCode::S,
        't' | 'T' => VirtualKeyCode::T,
        'u' | 'U' => VirtualKeyCode::U,
        'v' | 'V' => VirtualKeyCode::V,
        'w' | 'W' => VirtualKeyCode::W,
        'x' | 'X' => VirtualKeyCode::X,
        'y' | 'Y' => VirtualKeyCode::Y,
        'z' | 'Z' => VirtualKeyCode::Z,
        '1' | '!' => VirtualKeyCode::Key1,
        '2' | '@' => VirtualKeyCode::Key2,
        '3' | '#' => VirtualKeyCode::Key3,
        '4' | '$' => VirtualKeyCode::Key4,
        '5' | '%' => VirtualKeyCode::Key5,
        '6' | '^' => VirtualKeyCode::Key6,
        '7' | '&' => VirtualKeyCode::Key7,
        '8' | '*' => VirtualKeyCode::Key8,
        '9' | '(' => VirtualKeyCode::Key9,
        '0' | ')' => VirtualKeyCode::Key0,
        '=' | '+' => VirtualKeyCode::Equals,
        '-' | '_' => VirtualKeyCode::Minus,
        ']' | '}' => VirtualKeyCode::RBracket,
        '[' | '{' => VirtualKeyCode::LBracket,
        '\'' | '"' => VirtualKeyCode::Apostrophe,
        ';' | ':' => VirtualKeyCode::Semicolon,
        '\\' | '|' => VirtualKeyCode::Backslash,
        ',' | '<' => VirtualKeyCode::Comma,
        '/' | '?' => VirtualKeyCode::Slash,
        '.' | '>' => VirtualKeyCode::Period,
        '`' | '~' => VirtualKeyCode::Grave,
        _ => return None,
    })
}

bitflags::bitflags! {
    /// Represents the current state of the keyboard modifiers
    ///
    /// Each flag represents a modifier and is set if this modifier is active.
    #[derive(Default, Copy, Clone, Debug)]
    pub struct ModifiersState: u32 {
        // left and right modifiers are currently commented out, but we should be able to support
        // them in a future release
        /// The "shift" key.
        const SHIFT = 0b100;
        // const LSHIFT = 0b010;
        // const RSHIFT = 0b001;
        /// The "control" key.
        const CTRL = 0b100 << 3;
        // const LCTRL = 0b010 << 3;
        // const RCTRL = 0b001 << 3;
        /// The "alt" key.
        const ALT = 0b100 << 6;
        // const LALT = 0b010 << 6;
        // const RALT = 0b001 << 6;
        /// This is the "windows" key on PC and "command" key on Mac.
        const LOGO = 0b100 << 9;
        // const LLOGO = 0b010 << 9;
        // const RLOGO = 0b001 << 9;
    }
}
