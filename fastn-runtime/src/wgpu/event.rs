impl From<winit::event::Event<'_, ()>> for fastn_runtime::ExternalEvent {
    fn from(evt: winit::event::Event<()>) -> Self {
        dbg!(&evt);

        match evt {
            winit::event::Event::WindowEvent { event, .. } => match event {
                winit::event::WindowEvent::CursorMoved { position, .. } => {
                    fastn_runtime::ExternalEvent::CursorMoved {
                        x: position.x,
                        y: position.y,
                    }
                }
                winit::event::WindowEvent::Focused(f) => fastn_runtime::ExternalEvent::Focused(f),
                winit::event::WindowEvent::KeyboardInput { input, .. } => input.into(),
                winit::event::WindowEvent::ModifiersChanged(m) => {
                    fastn_runtime::ExternalEvent::ModifierChanged(
                        fastn_runtime::event::ModifiersState::from_bits_truncate(m.bits()),
                    )
                }
                _ => fastn_runtime::ExternalEvent::NoOp,
            },
            _ => fastn_runtime::ExternalEvent::NoOp,
        }
    }
}

impl From<winit::event::KeyboardInput> for fastn_runtime::ExternalEvent {
    fn from(evt: winit::event::KeyboardInput) -> Self {
        fastn_runtime::ExternalEvent::Key {
            pressed: match evt.state {
                winit::event::ElementState::Pressed => true,
                winit::event::ElementState::Released => false,
            },
            code: match evt.virtual_keycode {
                Some(v) => v.into(),
                None => return fastn_runtime::ExternalEvent::NoOp,
            },
        }
    }
}

impl From<winit::event::VirtualKeyCode> for fastn_runtime::event::VirtualKeyCode {
    fn from(v: winit::event::VirtualKeyCode) -> Self {
        match v {
            winit::event::VirtualKeyCode::Key1 => fastn_runtime::event::VirtualKeyCode::Key1,
            winit::event::VirtualKeyCode::Key2 => fastn_runtime::event::VirtualKeyCode::Key2,
            winit::event::VirtualKeyCode::Key3 => fastn_runtime::event::VirtualKeyCode::Key3,
            winit::event::VirtualKeyCode::Key4 => fastn_runtime::event::VirtualKeyCode::Key4,
            winit::event::VirtualKeyCode::Key5 => fastn_runtime::event::VirtualKeyCode::Key5,
            winit::event::VirtualKeyCode::Key6 => fastn_runtime::event::VirtualKeyCode::Key6,
            winit::event::VirtualKeyCode::Key7 => fastn_runtime::event::VirtualKeyCode::Key7,
            winit::event::VirtualKeyCode::Key8 => fastn_runtime::event::VirtualKeyCode::Key8,
            winit::event::VirtualKeyCode::Key9 => fastn_runtime::event::VirtualKeyCode::Key9,
            winit::event::VirtualKeyCode::Key0 => fastn_runtime::event::VirtualKeyCode::Key0,
            winit::event::VirtualKeyCode::A => fastn_runtime::event::VirtualKeyCode::A,
            winit::event::VirtualKeyCode::B => fastn_runtime::event::VirtualKeyCode::B,
            winit::event::VirtualKeyCode::C => fastn_runtime::event::VirtualKeyCode::C,
            winit::event::VirtualKeyCode::D => fastn_runtime::event::VirtualKeyCode::D,
            winit::event::VirtualKeyCode::E => fastn_runtime::event::VirtualKeyCode::E,
            winit::event::VirtualKeyCode::F => fastn_runtime::event::VirtualKeyCode::F,
            winit::event::VirtualKeyCode::G => fastn_runtime::event::VirtualKeyCode::G,
            winit::event::VirtualKeyCode::H => fastn_runtime::event::VirtualKeyCode::H,
            winit::event::VirtualKeyCode::I => fastn_runtime::event::VirtualKeyCode::I,
            winit::event::VirtualKeyCode::J => fastn_runtime::event::VirtualKeyCode::J,
            winit::event::VirtualKeyCode::K => fastn_runtime::event::VirtualKeyCode::K,
            winit::event::VirtualKeyCode::L => fastn_runtime::event::VirtualKeyCode::L,
            winit::event::VirtualKeyCode::M => fastn_runtime::event::VirtualKeyCode::M,
            winit::event::VirtualKeyCode::N => fastn_runtime::event::VirtualKeyCode::N,
            winit::event::VirtualKeyCode::O => fastn_runtime::event::VirtualKeyCode::O,
            winit::event::VirtualKeyCode::P => fastn_runtime::event::VirtualKeyCode::P,
            winit::event::VirtualKeyCode::Q => fastn_runtime::event::VirtualKeyCode::Q,
            winit::event::VirtualKeyCode::R => fastn_runtime::event::VirtualKeyCode::R,
            winit::event::VirtualKeyCode::S => fastn_runtime::event::VirtualKeyCode::S,
            winit::event::VirtualKeyCode::T => fastn_runtime::event::VirtualKeyCode::T,
            winit::event::VirtualKeyCode::U => fastn_runtime::event::VirtualKeyCode::U,
            winit::event::VirtualKeyCode::V => fastn_runtime::event::VirtualKeyCode::V,
            winit::event::VirtualKeyCode::W => fastn_runtime::event::VirtualKeyCode::W,
            winit::event::VirtualKeyCode::X => fastn_runtime::event::VirtualKeyCode::X,
            winit::event::VirtualKeyCode::Y => fastn_runtime::event::VirtualKeyCode::Y,
            winit::event::VirtualKeyCode::Z => fastn_runtime::event::VirtualKeyCode::Z,
            winit::event::VirtualKeyCode::Escape => fastn_runtime::event::VirtualKeyCode::Escape,
            winit::event::VirtualKeyCode::F1 => fastn_runtime::event::VirtualKeyCode::F1,
            winit::event::VirtualKeyCode::F2 => fastn_runtime::event::VirtualKeyCode::F2,
            winit::event::VirtualKeyCode::F3 => fastn_runtime::event::VirtualKeyCode::F3,
            winit::event::VirtualKeyCode::F4 => fastn_runtime::event::VirtualKeyCode::F4,
            winit::event::VirtualKeyCode::F5 => fastn_runtime::event::VirtualKeyCode::F5,
            winit::event::VirtualKeyCode::F6 => fastn_runtime::event::VirtualKeyCode::F6,
            winit::event::VirtualKeyCode::F7 => fastn_runtime::event::VirtualKeyCode::F7,
            winit::event::VirtualKeyCode::F8 => fastn_runtime::event::VirtualKeyCode::F8,
            winit::event::VirtualKeyCode::F9 => fastn_runtime::event::VirtualKeyCode::F9,
            winit::event::VirtualKeyCode::F10 => fastn_runtime::event::VirtualKeyCode::F10,
            winit::event::VirtualKeyCode::F11 => fastn_runtime::event::VirtualKeyCode::F11,
            winit::event::VirtualKeyCode::F12 => fastn_runtime::event::VirtualKeyCode::F12,
            winit::event::VirtualKeyCode::F13 => fastn_runtime::event::VirtualKeyCode::F13,
            winit::event::VirtualKeyCode::F14 => fastn_runtime::event::VirtualKeyCode::F14,
            winit::event::VirtualKeyCode::F15 => fastn_runtime::event::VirtualKeyCode::F15,
            winit::event::VirtualKeyCode::F16 => fastn_runtime::event::VirtualKeyCode::F16,
            winit::event::VirtualKeyCode::F17 => fastn_runtime::event::VirtualKeyCode::F17,
            winit::event::VirtualKeyCode::F18 => fastn_runtime::event::VirtualKeyCode::F18,
            winit::event::VirtualKeyCode::F19 => fastn_runtime::event::VirtualKeyCode::F19,
            winit::event::VirtualKeyCode::F20 => fastn_runtime::event::VirtualKeyCode::F20,
            winit::event::VirtualKeyCode::F21 => fastn_runtime::event::VirtualKeyCode::F21,
            winit::event::VirtualKeyCode::F22 => fastn_runtime::event::VirtualKeyCode::F22,
            winit::event::VirtualKeyCode::F23 => fastn_runtime::event::VirtualKeyCode::F23,
            winit::event::VirtualKeyCode::F24 => fastn_runtime::event::VirtualKeyCode::F24,
            winit::event::VirtualKeyCode::Snapshot => {
                fastn_runtime::event::VirtualKeyCode::Snapshot
            }
            winit::event::VirtualKeyCode::Scroll => fastn_runtime::event::VirtualKeyCode::Scroll,
            winit::event::VirtualKeyCode::Pause => fastn_runtime::event::VirtualKeyCode::Pause,
            winit::event::VirtualKeyCode::Insert => fastn_runtime::event::VirtualKeyCode::Insert,
            winit::event::VirtualKeyCode::Home => fastn_runtime::event::VirtualKeyCode::Home,
            winit::event::VirtualKeyCode::Delete => fastn_runtime::event::VirtualKeyCode::Delete,
            winit::event::VirtualKeyCode::End => fastn_runtime::event::VirtualKeyCode::End,
            winit::event::VirtualKeyCode::PageDown => {
                fastn_runtime::event::VirtualKeyCode::PageDown
            }
            winit::event::VirtualKeyCode::PageUp => fastn_runtime::event::VirtualKeyCode::PageUp,
            winit::event::VirtualKeyCode::Left => fastn_runtime::event::VirtualKeyCode::Left,
            winit::event::VirtualKeyCode::Up => fastn_runtime::event::VirtualKeyCode::Up,
            winit::event::VirtualKeyCode::Right => fastn_runtime::event::VirtualKeyCode::Right,
            winit::event::VirtualKeyCode::Down => fastn_runtime::event::VirtualKeyCode::Down,
            winit::event::VirtualKeyCode::Back => fastn_runtime::event::VirtualKeyCode::Back,
            winit::event::VirtualKeyCode::Return => fastn_runtime::event::VirtualKeyCode::Return,
            winit::event::VirtualKeyCode::Space => fastn_runtime::event::VirtualKeyCode::Space,
            winit::event::VirtualKeyCode::Compose => fastn_runtime::event::VirtualKeyCode::Compose,
            winit::event::VirtualKeyCode::Caret => fastn_runtime::event::VirtualKeyCode::Caret,
            winit::event::VirtualKeyCode::Numlock => fastn_runtime::event::VirtualKeyCode::Numlock,
            winit::event::VirtualKeyCode::Numpad0 => fastn_runtime::event::VirtualKeyCode::Numpad0,
            winit::event::VirtualKeyCode::Numpad1 => fastn_runtime::event::VirtualKeyCode::Numpad1,
            winit::event::VirtualKeyCode::Numpad2 => fastn_runtime::event::VirtualKeyCode::Numpad2,
            winit::event::VirtualKeyCode::Numpad3 => fastn_runtime::event::VirtualKeyCode::Numpad3,
            winit::event::VirtualKeyCode::Numpad4 => fastn_runtime::event::VirtualKeyCode::Numpad4,
            winit::event::VirtualKeyCode::Numpad5 => fastn_runtime::event::VirtualKeyCode::Numpad5,
            winit::event::VirtualKeyCode::Numpad6 => fastn_runtime::event::VirtualKeyCode::Numpad6,
            winit::event::VirtualKeyCode::Numpad7 => fastn_runtime::event::VirtualKeyCode::Numpad7,
            winit::event::VirtualKeyCode::Numpad8 => fastn_runtime::event::VirtualKeyCode::Numpad8,
            winit::event::VirtualKeyCode::Numpad9 => fastn_runtime::event::VirtualKeyCode::Numpad9,
            winit::event::VirtualKeyCode::NumpadAdd => {
                fastn_runtime::event::VirtualKeyCode::NumpadAdd
            }
            winit::event::VirtualKeyCode::NumpadDivide => {
                fastn_runtime::event::VirtualKeyCode::NumpadDivide
            }
            winit::event::VirtualKeyCode::NumpadDecimal => {
                fastn_runtime::event::VirtualKeyCode::NumpadDecimal
            }
            winit::event::VirtualKeyCode::NumpadComma => {
                fastn_runtime::event::VirtualKeyCode::NumpadComma
            }
            winit::event::VirtualKeyCode::NumpadEnter => {
                fastn_runtime::event::VirtualKeyCode::NumpadEnter
            }
            winit::event::VirtualKeyCode::NumpadEquals => {
                fastn_runtime::event::VirtualKeyCode::NumpadEquals
            }
            winit::event::VirtualKeyCode::NumpadMultiply => {
                fastn_runtime::event::VirtualKeyCode::NumpadMultiply
            }
            winit::event::VirtualKeyCode::NumpadSubtract => {
                fastn_runtime::event::VirtualKeyCode::NumpadSubtract
            }
            winit::event::VirtualKeyCode::AbntC1 => fastn_runtime::event::VirtualKeyCode::AbntC1,
            winit::event::VirtualKeyCode::AbntC2 => fastn_runtime::event::VirtualKeyCode::AbntC2,
            winit::event::VirtualKeyCode::Apostrophe => {
                fastn_runtime::event::VirtualKeyCode::Apostrophe
            }
            winit::event::VirtualKeyCode::Apps => fastn_runtime::event::VirtualKeyCode::Apps,
            winit::event::VirtualKeyCode::Asterisk => {
                fastn_runtime::event::VirtualKeyCode::Asterisk
            }
            winit::event::VirtualKeyCode::At => fastn_runtime::event::VirtualKeyCode::At,
            winit::event::VirtualKeyCode::Ax => fastn_runtime::event::VirtualKeyCode::Ax,
            winit::event::VirtualKeyCode::Backslash => {
                fastn_runtime::event::VirtualKeyCode::Backslash
            }
            winit::event::VirtualKeyCode::Calculator => {
                fastn_runtime::event::VirtualKeyCode::Calculator
            }
            winit::event::VirtualKeyCode::Capital => fastn_runtime::event::VirtualKeyCode::Capital,
            winit::event::VirtualKeyCode::Colon => fastn_runtime::event::VirtualKeyCode::Colon,
            winit::event::VirtualKeyCode::Comma => fastn_runtime::event::VirtualKeyCode::Comma,
            winit::event::VirtualKeyCode::Convert => fastn_runtime::event::VirtualKeyCode::Convert,
            winit::event::VirtualKeyCode::Equals => fastn_runtime::event::VirtualKeyCode::Equals,
            winit::event::VirtualKeyCode::Grave => fastn_runtime::event::VirtualKeyCode::Grave,
            winit::event::VirtualKeyCode::Kana => fastn_runtime::event::VirtualKeyCode::Kana,
            winit::event::VirtualKeyCode::Kanji => fastn_runtime::event::VirtualKeyCode::Kanji,
            winit::event::VirtualKeyCode::LAlt => fastn_runtime::event::VirtualKeyCode::LAlt,
            winit::event::VirtualKeyCode::LBracket => {
                fastn_runtime::event::VirtualKeyCode::LBracket
            }
            winit::event::VirtualKeyCode::LControl => {
                fastn_runtime::event::VirtualKeyCode::LControl
            }
            winit::event::VirtualKeyCode::LShift => fastn_runtime::event::VirtualKeyCode::LShift,
            winit::event::VirtualKeyCode::LWin => fastn_runtime::event::VirtualKeyCode::LWin,
            winit::event::VirtualKeyCode::Mail => fastn_runtime::event::VirtualKeyCode::Mail,
            winit::event::VirtualKeyCode::MediaSelect => {
                fastn_runtime::event::VirtualKeyCode::MediaSelect
            }
            winit::event::VirtualKeyCode::MediaStop => {
                fastn_runtime::event::VirtualKeyCode::MediaStop
            }
            winit::event::VirtualKeyCode::Minus => fastn_runtime::event::VirtualKeyCode::Minus,
            winit::event::VirtualKeyCode::Mute => fastn_runtime::event::VirtualKeyCode::Mute,
            winit::event::VirtualKeyCode::MyComputer => {
                fastn_runtime::event::VirtualKeyCode::MyComputer
            }
            winit::event::VirtualKeyCode::NavigateForward => {
                fastn_runtime::event::VirtualKeyCode::NavigateForward
            }
            winit::event::VirtualKeyCode::NavigateBackward => {
                fastn_runtime::event::VirtualKeyCode::NavigateBackward
            }
            winit::event::VirtualKeyCode::NextTrack => {
                fastn_runtime::event::VirtualKeyCode::NextTrack
            }
            winit::event::VirtualKeyCode::NoConvert => {
                fastn_runtime::event::VirtualKeyCode::NoConvert
            }
            winit::event::VirtualKeyCode::OEM102 => fastn_runtime::event::VirtualKeyCode::OEM102,
            winit::event::VirtualKeyCode::Period => fastn_runtime::event::VirtualKeyCode::Period,
            winit::event::VirtualKeyCode::PlayPause => {
                fastn_runtime::event::VirtualKeyCode::PlayPause
            }
            winit::event::VirtualKeyCode::Plus => fastn_runtime::event::VirtualKeyCode::Plus,
            winit::event::VirtualKeyCode::Power => fastn_runtime::event::VirtualKeyCode::Power,
            winit::event::VirtualKeyCode::PrevTrack => {
                fastn_runtime::event::VirtualKeyCode::PrevTrack
            }
            winit::event::VirtualKeyCode::RAlt => fastn_runtime::event::VirtualKeyCode::RAlt,
            winit::event::VirtualKeyCode::RBracket => {
                fastn_runtime::event::VirtualKeyCode::RBracket
            }
            winit::event::VirtualKeyCode::RControl => {
                fastn_runtime::event::VirtualKeyCode::RControl
            }
            winit::event::VirtualKeyCode::RShift => fastn_runtime::event::VirtualKeyCode::RShift,
            winit::event::VirtualKeyCode::RWin => fastn_runtime::event::VirtualKeyCode::RWin,
            winit::event::VirtualKeyCode::Semicolon => {
                fastn_runtime::event::VirtualKeyCode::Semicolon
            }
            winit::event::VirtualKeyCode::Slash => fastn_runtime::event::VirtualKeyCode::Slash,
            winit::event::VirtualKeyCode::Sleep => fastn_runtime::event::VirtualKeyCode::Sleep,
            winit::event::VirtualKeyCode::Stop => fastn_runtime::event::VirtualKeyCode::Stop,
            winit::event::VirtualKeyCode::Sysrq => fastn_runtime::event::VirtualKeyCode::Sysrq,
            winit::event::VirtualKeyCode::Tab => fastn_runtime::event::VirtualKeyCode::Tab,
            winit::event::VirtualKeyCode::Underline => {
                fastn_runtime::event::VirtualKeyCode::Underline
            }
            winit::event::VirtualKeyCode::Unlabeled => {
                fastn_runtime::event::VirtualKeyCode::Unlabeled
            }
            winit::event::VirtualKeyCode::VolumeDown => {
                fastn_runtime::event::VirtualKeyCode::VolumeDown
            }
            winit::event::VirtualKeyCode::VolumeUp => {
                fastn_runtime::event::VirtualKeyCode::VolumeUp
            }
            winit::event::VirtualKeyCode::Wake => fastn_runtime::event::VirtualKeyCode::Wake,
            winit::event::VirtualKeyCode::WebBack => fastn_runtime::event::VirtualKeyCode::WebBack,
            winit::event::VirtualKeyCode::WebFavorites => {
                fastn_runtime::event::VirtualKeyCode::WebFavorites
            }
            winit::event::VirtualKeyCode::WebForward => {
                fastn_runtime::event::VirtualKeyCode::WebForward
            }
            winit::event::VirtualKeyCode::WebHome => fastn_runtime::event::VirtualKeyCode::WebHome,
            winit::event::VirtualKeyCode::WebRefresh => {
                fastn_runtime::event::VirtualKeyCode::WebRefresh
            }
            winit::event::VirtualKeyCode::WebSearch => {
                fastn_runtime::event::VirtualKeyCode::WebSearch
            }
            winit::event::VirtualKeyCode::WebStop => fastn_runtime::event::VirtualKeyCode::WebStop,
            winit::event::VirtualKeyCode::Yen => fastn_runtime::event::VirtualKeyCode::Yen,
            winit::event::VirtualKeyCode::Copy => fastn_runtime::event::VirtualKeyCode::Copy,
            winit::event::VirtualKeyCode::Paste => fastn_runtime::event::VirtualKeyCode::Paste,
            winit::event::VirtualKeyCode::Cut => fastn_runtime::event::VirtualKeyCode::Cut,
        }
    }
}
