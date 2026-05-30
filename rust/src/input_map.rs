use keyboard_types::Key;

pub fn map_key(code: i32) -> Key {
    use keyboard_types::Key as K;
    match code {
        65..=90 => Key::Character(char::from_u32((code + 32) as u32).unwrap_or(' ')),
        48..=57 => Key::Character(char::from_u32(code as u32).unwrap_or(' ')),
        4194305 => K::Backspace,
        4194306 => K::Tab,
        4194307 => K::Enter,
        4194308 => K::Shift,
        4194309 => K::Control,
        4194310 => K::Alt,
        4194311 => K::Escape,
        4194318 => K::ArrowLeft,
        4194319 => K::ArrowUp,
        4194320 => K::ArrowRight,
        4194321 => K::ArrowDown,
        4194322 => K::Home,
        4194323 => K::End,
        4194324 => K::PageUp,
        4194325 => K::PageDown,
        4194337 => K::Delete,
        _ => Key::Character(' '),
    }
}
