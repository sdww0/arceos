//! Convert keyboard input to ASCII, such as Ctrl+C input to ASCII code 3

use ascii::AsciiChar;

use super::key::{InputEvent, Key, KeyStatus};

/// Convert keyboard input to ASCII.
pub struct KeyToAscii {
    /// Left ctrl, Right ctrl status
    ctrl_status: [KeyStatus; 2],
    /// Left shift, Right shift status
    shift_status: [KeyStatus; 2],
    /// Caps lock status
    caps_lock_status: KeyStatus,
}

impl KeyToAscii {
    pub const fn new() -> Self {
        Self {
            ctrl_status: [KeyStatus::Released, KeyStatus::Released],
            shift_status: [KeyStatus::Released, KeyStatus::Released],
            caps_lock_status: KeyStatus::Released,
        }
    }

    /// Convert input event to ascii. If the event status is released or the output is unknown, then the return value is `None`.
    pub fn input(&mut self, event: InputEvent) -> Option<[AsciiChar; 4]> {
        let InputEvent::KeyBoard(key, status) = event else {
            return None;
        };
        match key {
            Key::LeftShift => {
                self.shift_status[0] = status;
                return None;
            }
            Key::RightShift => {
                self.shift_status[1] = status;
                return None;
            }
            Key::LeftCtrl => {
                self.ctrl_status[0] = status;
                return None;
            }
            Key::RightCtrl => {
                self.ctrl_status[1] = status;
                return None;
            }
            Key::Capslock => {
                if status == KeyStatus::Pressed {
                    self.caps_lock_status = match self.caps_lock_status {
                        KeyStatus::Pressed => KeyStatus::Released,
                        KeyStatus::Released => KeyStatus::Pressed,
                    }
                }
                return None;
            }
            _ => {}
        };
        if status == KeyStatus::Released {
            return None;
        }

        if key == Key::Up {
            return Some([
                AsciiChar::ESC,
                AsciiChar::BracketOpen,
                AsciiChar::A,
                AsciiChar::Null,
            ]);
        } else if key == Key::Down {
            return Some([
                AsciiChar::ESC,
                AsciiChar::BracketOpen,
                AsciiChar::B,
                AsciiChar::Null,
            ]);
        } else if key == Key::Right {
            return Some([
                AsciiChar::ESC,
                AsciiChar::BracketOpen,
                AsciiChar::C,
                AsciiChar::Null,
            ]);
        } else if key == Key::Left {
            return Some([
                AsciiChar::ESC,
                AsciiChar::BracketOpen,
                AsciiChar::D,
                AsciiChar::Null,
            ]);
        }

        let is_keypad = match key {
            Key::Kp0
            | Key::Kp1
            | Key::Kp2
            | Key::Kp3
            | Key::Kp4
            | Key::Kp5
            | Key::Kp6
            | Key::Kp7
            | Key::Kp8
            | Key::Kp9
            | Key::KpAsterisk
            | Key::KpDot
            | Key::KpEnter
            | Key::KpMinus
            | Key::KpPlus
            | Key::KpSlash => true,
            _ => false,
        };
        let mut ascii = match key {
            Key::Enter => AsciiChar::CarriageReturn,
            Key::Tab => AsciiChar::Tab,
            Key::BackSpace => AsciiChar::BackSpace,
            Key::Space => AsciiChar::Space,
            Key::Apostrophe => AsciiChar::Apostrophe,
            Key::KpAsterisk => AsciiChar::Asterisk,
            Key::KpPlus => AsciiChar::Plus,
            Key::Comma => AsciiChar::Comma,
            Key::KpMinus | Key::Minus => AsciiChar::Minus,
            Key::KpDot | Key::Dot => AsciiChar::Dot,
            Key::KpSlash | Key::Slash => AsciiChar::Slash,
            Key::LeftBrace => AsciiChar::BracketOpen,
            Key::RightBrace => AsciiChar::BracketClose,
            Key::BackSlash => AsciiChar::BackSlash,
            
            Key::Zero => AsciiChar::_0,
            Key::One => AsciiChar::_1,
            Key::Two => AsciiChar::_2,
            Key::Three => AsciiChar::_3,
            Key::Four => AsciiChar::_4,
            Key::Five => AsciiChar::_5,
            Key::Six => AsciiChar::_6,
            Key::Seven => AsciiChar::_7,
            Key::Eight => AsciiChar::_8,
            Key::Nine => AsciiChar::_9,

            Key::SemiColon => AsciiChar::Semicolon,
            Key::Equal => AsciiChar::Equal,

            Key::Grave => AsciiChar::Grave,
            Key::A => AsciiChar::a,
            Key::B => AsciiChar::b,
            Key::C => AsciiChar::c,
            Key::D => AsciiChar::d,
            Key::E => AsciiChar::e,
            Key::F => AsciiChar::f,
            Key::G => AsciiChar::g,
            Key::H => AsciiChar::h,
            Key::I => AsciiChar::i,
            Key::J => AsciiChar::j,
            Key::K => AsciiChar::k,
            Key::L => AsciiChar::l,
            Key::M => AsciiChar::m,
            Key::N => AsciiChar::n,
            Key::O => AsciiChar::o,
            Key::P => AsciiChar::p,
            Key::Q => AsciiChar::q,
            Key::R => AsciiChar::r,
            Key::S => AsciiChar::s,
            Key::T => AsciiChar::t,
            Key::U => AsciiChar::u,
            Key::V => AsciiChar::v,
            Key::W => AsciiChar::w,
            Key::X => AsciiChar::x,
            Key::Y => AsciiChar::y,
            Key::Z => AsciiChar::z,
            _ => return None,
        };

        if is_keypad {
            return Some([ascii, AsciiChar::Null, AsciiChar::Null, AsciiChar::Null]);
        }

        // check caps lock
        if self.caps_lock_status == KeyStatus::Pressed {
            ascii.make_ascii_uppercase();
        }

        if self.shift_status[0] == KeyStatus::Pressed || self.shift_status[1] == KeyStatus::Pressed
        {
            // Shift
            if ascii.is_ascii_lowercase() {
                ascii.make_ascii_uppercase();
            } else if ascii.is_ascii_uppercase() {
                ascii.make_ascii_lowercase();
            }
            ascii = match ascii {
                AsciiChar::Grave => AsciiChar::Tilde,
                AsciiChar::_1 => AsciiChar::Exclamation,
                AsciiChar::_2 => AsciiChar::At,
                AsciiChar::_3 => AsciiChar::Hash,
                AsciiChar::_4 => AsciiChar::Dollar,
                AsciiChar::_5 => AsciiChar::Percent,
                AsciiChar::_6 => AsciiChar::Caret,
                AsciiChar::_7 => AsciiChar::Ampersand,
                AsciiChar::_8 => AsciiChar::Asterisk,
                AsciiChar::_9 => AsciiChar::ParenOpen,
                AsciiChar::_0 => AsciiChar::ParenClose,
                AsciiChar::Minus => AsciiChar::UnderScore,
                AsciiChar::Equal => AsciiChar::Plus,
                AsciiChar::BracketOpen => AsciiChar::CurlyBraceOpen,
                AsciiChar::BracketClose => AsciiChar::CurlyBraceClose,
                AsciiChar::BackSlash => AsciiChar::VerticalBar,
                AsciiChar::Semicolon => AsciiChar::Colon,
                AsciiChar::Apostrophe => AsciiChar::Quotation,
                AsciiChar::Comma => AsciiChar::LessThan,
                AsciiChar::Dot => AsciiChar::GreaterThan,
                AsciiChar::Slash => AsciiChar::Question,
                _ => ascii,
            }
        } else if self.ctrl_status[0] == KeyStatus::Pressed
            || self.ctrl_status[1] == KeyStatus::Pressed
        {
            // Ctrl
            ascii = if ascii.is_ascii_uppercase() {
                AsciiChar::from_ascii(ascii as u8 - AsciiChar::A as u8 + 1).unwrap()
            } else if ascii.is_ascii_lowercase() {
                AsciiChar::from_ascii(ascii as u8 - AsciiChar::a as u8 + 1).unwrap()
            } else {
                match ascii {
                    AsciiChar::BracketOpen => AsciiChar::ESC,
                    AsciiChar::BackSlash => AsciiChar::FS,
                    AsciiChar::BracketClose => AsciiChar::GS,
                    AsciiChar::_6 => AsciiChar::RS,
                    AsciiChar::Minus => AsciiChar::US,
                    _ => ascii,
                }
            }
        }
        Some([ascii, AsciiChar::Null, AsciiChar::Null, AsciiChar::Null])
    }
}
