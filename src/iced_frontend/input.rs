use alacritty_terminal::term::TermMode;
use iced::keyboard::{Event, Key, key::Named};

pub(super) fn encode(event: &Event, mode: TermMode) -> Option<Vec<u8>> {
    let Event::KeyPressed {
        key,
        modified_key,
        modifiers,
        text,
        ..
    } = event
    else {
        return None;
    };
    if modifiers.logo() {
        return None;
    }

    let bytes: &[u8] = match key.as_ref() {
        Key::Named(Named::Enter) => b"\r",
        Key::Named(Named::Tab) if modifiers.shift() => b"\x1b[Z",
        Key::Named(Named::Tab) => b"\t",
        Key::Named(Named::Backspace) if modifiers.control() => b"\x08",
        Key::Named(Named::Backspace) => b"\x7f",
        Key::Named(Named::Escape) => b"\x1b",
        Key::Named(
            key @ (Named::ArrowUp
            | Named::ArrowDown
            | Named::ArrowRight
            | Named::ArrowLeft
            | Named::Home
            | Named::End),
        ) => {
            let code = match key {
                Named::ArrowUp => b'A',
                Named::ArrowDown => b'B',
                Named::ArrowRight => b'C',
                Named::ArrowLeft => b'D',
                Named::Home => b'H',
                Named::End => b'F',
                _ => unreachable!(),
            };
            let modifier = 1
                + u8::from(modifiers.shift())
                + u8::from(modifiers.alt()) * 2
                + u8::from(modifiers.control()) * 4;
            return Some(if modifier != 1 {
                format!("\x1b[1;{modifier}{}", char::from(code)).into_bytes()
            } else if mode.contains(TermMode::APP_CURSOR) {
                vec![0x1b, b'O', code]
            } else {
                vec![0x1b, b'[', code]
            });
        }
        Key::Named(Named::Insert) => b"\x1b[2~",
        Key::Named(Named::Delete) => b"\x1b[3~",
        Key::Named(Named::PageUp) => b"\x1b[5~",
        Key::Named(Named::PageDown) => b"\x1b[6~",
        Key::Named(Named::F1) => b"\x1bOP",
        Key::Named(Named::F2) => b"\x1bOQ",
        Key::Named(Named::F3) => b"\x1bOR",
        Key::Named(Named::F4) => b"\x1bOS",
        Key::Named(Named::F5) => b"\x1b[15~",
        Key::Named(Named::F6) => b"\x1b[17~",
        Key::Named(Named::F7) => b"\x1b[18~",
        Key::Named(Named::F8) => b"\x1b[19~",
        Key::Named(Named::F9) => b"\x1b[20~",
        Key::Named(Named::F10) => b"\x1b[21~",
        Key::Named(Named::F11) => b"\x1b[23~",
        Key::Named(Named::F12) => b"\x1b[24~",
        Key::Named(Named::Space) if modifiers.control() => b"\0",
        Key::Named(_) => text.as_ref()?.as_bytes(),
        Key::Character(_) if modifiers.control() => {
            let Key::Character(character) = modified_key.as_ref() else {
                return None;
            };
            let byte = match character.as_bytes() {
                [byte @ (b'a'..=b'z' | b'@'..=b'_')] => byte & 0x1f,
                [b'?'] => 0x7f,
                _ => return None,
            };
            return Some(with_alt(&[byte], modifiers.alt()));
        }
        _ => text.as_ref()?.as_bytes(),
    };
    Some(with_alt(bytes, modifiers.alt()))
}

fn with_alt(bytes: &[u8], alt: bool) -> Vec<u8> {
    let mut sequence = Vec::with_capacity(bytes.len() + usize::from(alt));
    if alt {
        sequence.push(0x1b);
    }
    sequence.extend_from_slice(bytes);
    sequence
}

#[cfg(test)]
mod tests {
    use super::*;
    use iced::keyboard::{Location, Modifiers, key};

    fn press(key: Key, modifiers: Modifiers) -> Event {
        Event::KeyPressed {
            modified_key: key.clone(),
            key,
            physical_key: key::Physical::Unidentified(key::NativeCode::Unidentified),
            location: Location::Standard,
            modifiers,
            text: None,
            repeat: false,
        }
    }

    #[test]
    fn encodes_interrupt_and_application_cursor_input() {
        let ctrl_c = press(Key::Character("c".into()), Modifiers::CTRL);
        assert_eq!(encode(&ctrl_c, TermMode::empty()), Some(vec![3]));
        let arrow = press(Key::Named(Named::ArrowUp), Modifiers::empty());
        assert_eq!(
            encode(&arrow, TermMode::APP_CURSOR),
            Some(b"\x1bOA".to_vec())
        );
        assert_eq!(encode(&arrow, TermMode::empty()), Some(b"\x1b[A".to_vec()));
    }
}
