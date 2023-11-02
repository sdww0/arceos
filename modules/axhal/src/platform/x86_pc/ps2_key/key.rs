use num_enum::TryFromPrimitive;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, TryFromPrimitive)]
#[repr(u16)]
pub enum Key {
    Reserved = 0,
    ESC = 1,
    One = 2,
    Two = 3,
    Three = 4,
    Four = 5,
    Five = 6,
    Six = 7,
    Seven = 8,
    Eight = 9,
    Nine = 10,
    Zero = 11,
    Minus = 12,
    Equal = 13,
    BackSpace = 14,
    Tab = 15,
    Q = 16,
    W = 17,
    E = 18,
    R = 19,
    T = 20,
    Y = 21,
    U = 22,
    I = 23,
    O = 24,
    P = 25,
    /// Symbol: [
    LeftBrace = 26,
    /// Symbol: ]
    RightBrace = 27,
    Enter = 28,
    LeftCtrl = 29,
    A = 30,
    S = 31,
    D = 32,
    F = 33,
    G = 34,
    H = 35,
    J = 36,
    K = 37,
    L = 38,
    /// Symbol: ;
    SemiColon = 39,
    /// Symbol: '
    Apostrophe = 40,
    /// Symbol: `
    Grave = 41,
    LeftShift = 42,
    /// Symbol: \
    BackSlash = 43,
    Z = 44,
    X = 45,
    C = 46,
    V = 47,
    B = 48,
    N = 49,
    M = 50,
    Comma = 51,
    Dot = 52,
    // Symbol: /
    Slash = 53,
    RightShift = 54,
    /// Keypad asterisk, Symbol: *
    KpAsterisk = 55,
    LeftAlt = 56,
    Space = 57,
    Capslock = 58,
    F1 = 59,
    F2 = 60,
    F3 = 61,
    F4 = 62,
    F5 = 63,
    F6 = 64,
    F7 = 65,
    F8 = 66,
    F9 = 67,
    F10 = 68,
    NumLock = 69,
    ScrollLock = 70,
    Kp7 = 71,
    Kp8 = 72,
    Kp9 = 73,
    KpMinus = 74,
    Kp4 = 75,
    Kp5 = 76,
    Kp6 = 77,
    KpPlus = 78,
    Kp1 = 79,
    Kp2 = 80,
    Kp3 = 81,
    Kp0 = 82,
    KpDot = 83,

    F11 = 87,
    F12 = 88,

    KpEnter = 96,
    RightCtrl = 97,
    KpSlash = 98,

    RightAlt = 100,
    LineFeed = 101,
    Home = 102,
    Up = 103,
    PageUp = 104,
    Left = 105,
    Right = 106,
    End = 107,
    Down = 108,
    PageDown = 109,
    Insert = 110,
    Delete = 111,

    LeftMeta = 125,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyStatus {
    Pressed,
    Released,
}

#[derive(Debug, Clone, Copy)]
pub enum InputEvent {
    KeyBoard(Key, KeyStatus),
}
