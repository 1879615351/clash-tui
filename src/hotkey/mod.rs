/// Hotkey module — placeholder for Windows global hotkey support.
///
/// In Phase 3, the hotkey toggles TUI visibility via Ctrl+`.
/// Implementation uses RegisterHotKey with a background message pump on Windows.

/// Parse a hotkey string like "Ctrl+`" into (modifier_flags, virtual_key_code).
pub fn parse_hotkey(s: &str) -> Option<(u32, u32)> {
    let parts: Vec<&str> = s.split('+').map(|p| p.trim()).collect();
    if parts.len() < 2 {
        return None;
    }

    let mut mod_key = 0u32;
    for part in &parts[..parts.len() - 1] {
        match part.to_lowercase().as_str() {
            "ctrl" | "control" => mod_key |= 0x0002,
            "alt" => mod_key |= 0x0001,
            "shift" => mod_key |= 0x0004,
            "win" | "windows" => mod_key |= 0x0008,
            _ => {}
        }
    }

    let key = parts.last()?.to_lowercase();
    let vk = match key.as_str() {
        "`" | "~" | "oem3" => 0xC0u32,
        "f1" => 0x70,
        "f2" => 0x71,
        "f3" => 0x72,
        "f4" => 0x73,
        "f5" => 0x74,
        "f6" => 0x75,
        "f7" => 0x76,
        "f8" => 0x77,
        "f9" => 0x78,
        "f10" => 0x79,
        "f11" => 0x7A,
        "f12" => 0x7B,
        "space" => 0x20,
        "tab" => 0x09,
        "escape" | "esc" => 0x1B,
        c if c.len() == 1 => c.chars().next()?.to_ascii_uppercase() as u32,
        _ => return None,
    };

    Some((mod_key, vk))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_hotkey() {
        let (mod_, vk) = parse_hotkey("Ctrl+`").unwrap();
        assert_eq!(mod_, 0x0002);
        assert_eq!(vk, 0xC0);

        let (mod_, vk) = parse_hotkey("Alt+F4").unwrap();
        assert_eq!(mod_, 0x0001);
        assert_eq!(vk, 0x73);

        let (mod_, vk) = parse_hotkey("Ctrl+Shift+A").unwrap();
        assert_eq!(mod_, 0x0002 | 0x0004);
        assert_eq!(vk, 'A' as u32);
    }
}
