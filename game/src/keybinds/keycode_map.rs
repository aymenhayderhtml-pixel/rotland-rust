//! Two-way mapping between Godot key codes (including Shift modifier mask) and winit KeyCode.

use winit::keyboard::KeyCode;

/// Godot's Shift modifier bit (KeyModifierMask.ShiftKey = 1 << 25).
// Godot: KeyBindings.cs ShiftMask
pub const KEYBINDS_SHIFT_MASK: u32 = 1 << 25;

/// Mask for the key code without the Shift bit.
// Godot: KeyBindings.cs CodeMask
pub const KEYBINDS_CODE_MASK: u32 = KEYBINDS_SHIFT_MASK - 1;

/// Maps a Godot integer key code (with optional ShiftMask bit) to winit `KeyCode` and a shift flag.
pub fn godot_code_to_key(code: u32) -> Option<(KeyCode, bool)> {
    if code == 0 {
        return None;
    }
    let shift = (code & KEYBINDS_SHIFT_MASK) != 0;
    let base_code = code & KEYBINDS_CODE_MASK;
    let key = match base_code {
        32 => KeyCode::Space,
        65 => KeyCode::KeyA,
        66 => KeyCode::KeyB,
        67 => KeyCode::KeyC,
        68 => KeyCode::KeyD,
        69 => KeyCode::KeyE,
        70 => KeyCode::KeyF,
        71 => KeyCode::KeyG,
        72 => KeyCode::KeyH,
        73 => KeyCode::KeyI,
        74 => KeyCode::KeyJ,
        75 => KeyCode::KeyK,
        76 => KeyCode::KeyL,
        77 => KeyCode::KeyM,
        78 => KeyCode::KeyN,
        79 => KeyCode::KeyO,
        80 => KeyCode::KeyP,
        81 => KeyCode::KeyQ,
        82 => KeyCode::KeyR,
        83 => KeyCode::KeyS,
        84 => KeyCode::KeyT,
        85 => KeyCode::KeyU,
        86 => KeyCode::KeyV,
        87 => KeyCode::KeyW,
        88 => KeyCode::KeyX,
        89 => KeyCode::KeyY,
        90 => KeyCode::KeyZ,
        48 => KeyCode::Digit0,
        49 => KeyCode::Digit1,
        50 => KeyCode::Digit2,
        51 => KeyCode::Digit3,
        52 => KeyCode::Digit4,
        53 => KeyCode::Digit5,
        54 => KeyCode::Digit6,
        55 => KeyCode::Digit7,
        56 => KeyCode::Digit8,
        57 => KeyCode::Digit9,
        4194305 => KeyCode::Escape,
        4194306 => KeyCode::Tab,
        4194308 => KeyCode::Backspace,
        4194309 => KeyCode::Enter,
        4194314 => KeyCode::Insert,
        4194315 => KeyCode::Delete,
        4194317 => KeyCode::Home,
        4194318 => KeyCode::End,
        4194319 => KeyCode::ArrowLeft,
        4194320 => KeyCode::ArrowUp,
        4194321 => KeyCode::ArrowRight,
        4194322 => KeyCode::ArrowDown,
        4194323 => KeyCode::PageUp,
        4194324 => KeyCode::PageDown,
        4194325 => KeyCode::ShiftLeft,
        4194326 => KeyCode::ControlLeft,
        4194328 => KeyCode::AltLeft,
        4194329 => KeyCode::CapsLock,
        4194332 => KeyCode::F1,
        4194333 => KeyCode::F2,
        4194334 => KeyCode::F3,
        4194335 => KeyCode::F4,
        4194336 => KeyCode::F5,
        4194337 => KeyCode::F6,
        4194338 => KeyCode::F7,
        4194339 => KeyCode::F8,
        4194340 => KeyCode::F9,
        4194341 => KeyCode::F10,
        4194342 => KeyCode::F11,
        4194343 => KeyCode::F12,
        45 => KeyCode::Minus,
        61 => KeyCode::Equal,
        91 => KeyCode::BracketLeft,
        93 => KeyCode::BracketRight,
        59 => KeyCode::Semicolon,
        39 => KeyCode::Quote,
        96 => KeyCode::Backquote,
        92 => KeyCode::Backslash,
        44 => KeyCode::Comma,
        46 => KeyCode::Period,
        47 => KeyCode::Slash,
        _ => return None,
    };
    Some((key, shift))
}

/// Maps a winit `KeyCode` and optional shift flag to a Godot integer key code.
pub fn key_to_godot_code(key: KeyCode, shift: bool) -> Option<u32> {
    let base: u32 = match key {
        KeyCode::Space => 32,
        KeyCode::KeyA => 65,
        KeyCode::KeyB => 66,
        KeyCode::KeyC => 67,
        KeyCode::KeyD => 68,
        KeyCode::KeyE => 69,
        KeyCode::KeyF => 70,
        KeyCode::KeyG => 71,
        KeyCode::KeyH => 72,
        KeyCode::KeyI => 73,
        KeyCode::KeyJ => 74,
        KeyCode::KeyK => 75,
        KeyCode::KeyL => 76,
        KeyCode::KeyM => 77,
        KeyCode::KeyN => 78,
        KeyCode::KeyO => 79,
        KeyCode::KeyP => 80,
        KeyCode::KeyQ => 81,
        KeyCode::KeyR => 82,
        KeyCode::KeyS => 83,
        KeyCode::KeyT => 84,
        KeyCode::KeyU => 85,
        KeyCode::KeyV => 86,
        KeyCode::KeyW => 87,
        KeyCode::KeyX => 88,
        KeyCode::KeyY => 89,
        KeyCode::KeyZ => 90,
        KeyCode::Digit0 => 48,
        KeyCode::Digit1 => 49,
        KeyCode::Digit2 => 50,
        KeyCode::Digit3 => 51,
        KeyCode::Digit4 => 52,
        KeyCode::Digit5 => 53,
        KeyCode::Digit6 => 54,
        KeyCode::Digit7 => 55,
        KeyCode::Digit8 => 56,
        KeyCode::Digit9 => 57,
        KeyCode::Escape => 4194305,
        KeyCode::Tab => 4194306,
        KeyCode::Backspace => 4194308,
        KeyCode::Enter => 4194309,
        KeyCode::Insert => 4194314,
        KeyCode::Delete => 4194315,
        KeyCode::Home => 4194317,
        KeyCode::End => 4194318,
        KeyCode::ArrowLeft => 4194319,
        KeyCode::ArrowUp => 4194320,
        KeyCode::ArrowRight => 4194321,
        KeyCode::ArrowDown => 4194322,
        KeyCode::PageUp => 4194323,
        KeyCode::PageDown => 4194324,
        KeyCode::ShiftLeft | KeyCode::ShiftRight => 4194325,
        KeyCode::ControlLeft | KeyCode::ControlRight => 4194326,
        KeyCode::AltLeft | KeyCode::AltRight => 4194328,
        KeyCode::CapsLock => 4194329,
        KeyCode::F1 => 4194332,
        KeyCode::F2 => 4194333,
        KeyCode::F3 => 4194334,
        KeyCode::F4 => 4194335,
        KeyCode::F5 => 4194336,
        KeyCode::F6 => 4194337,
        KeyCode::F7 => 4194338,
        KeyCode::F8 => 4194339,
        KeyCode::F9 => 4194340,
        KeyCode::F10 => 4194341,
        KeyCode::F11 => 4194342,
        KeyCode::F12 => 4194343,
        KeyCode::Minus => 45,
        KeyCode::Equal => 61,
        KeyCode::BracketLeft => 91,
        KeyCode::BracketRight => 93,
        KeyCode::Semicolon => 59,
        KeyCode::Quote => 39,
        KeyCode::Backquote => 96,
        KeyCode::Backslash => 92,
        KeyCode::Comma => 44,
        KeyCode::Period => 46,
        KeyCode::Slash => 47,
        _ => return None,
    };
    let is_shift_key = matches!(key, KeyCode::ShiftLeft | KeyCode::ShiftRight);
    let code = if shift && !is_shift_key {
        base | KEYBINDS_SHIFT_MASK
    } else {
        base
    };
    Some(code)
}

/// Text representation for a Godot key code, such as "E", "Shift+G" or "Space".
// Godot: KeyBindings.cs Describe
pub fn describe_godot_code(code: u32) -> String {
    if code == 0 {
        return String::new();
    }
    let has_shift = (code & KEYBINDS_SHIFT_MASK) != 0;
    let base = code & KEYBINDS_CODE_MASK;
    let name = match base {
        32 => "Space",
        65..=90 => {
            let ch = (base as u8) as char;
            return if has_shift {
                format!("Shift+{ch}")
            } else {
                ch.to_string()
            };
        }
        48..=57 => {
            let ch = (base as u8) as char;
            return if has_shift {
                format!("Shift+{ch}")
            } else {
                ch.to_string()
            };
        }
        4194305 => "Escape",
        4194306 => "Tab",
        4194308 => "Backspace",
        4194309 => "Enter",
        4194314 => "Insert",
        4194315 => "Delete",
        4194317 => "Home",
        4194318 => "End",
        4194319 => "Left",
        4194320 => "Up",
        4194321 => "Right",
        4194322 => "Down",
        4194323 => "PageUp",
        4194324 => "PageDown",
        4194325 => "Shift",
        4194326 => "Ctrl",
        4194328 => "Alt",
        4194329 => "CapsLock",
        4194332 => "F1",
        4194333 => "F2",
        4194334 => "F3",
        4194335 => "F4",
        4194336 => "F5",
        4194337 => "F6",
        4194338 => "F7",
        4194339 => "F8",
        4194340 => "F9",
        4194341 => "F10",
        4194342 => "F11",
        4194343 => "F12",
        45 => "-",
        61 => "=",
        91 => "[",
        93 => "]",
        59 => ";",
        39 => "'",
        96 => "`",
        92 => "\\",
        44 => ",",
        46 => ".",
        47 => "/",
        _ => "Unknown",
    };
    if has_shift && base != 4194325 {
        format!("Shift+{name}")
    } else {
        name.to_string()
    }
}
