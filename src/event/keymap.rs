use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::action::Action;

/// Global keymapping: convert KeyEvent to Actions
/// These run BEFORE page-level handlers and always win.
pub fn map_key(key: KeyEvent) -> Vec<Action> {
    match key.code {
        // Quit
        KeyCode::Char('q') if key.modifiers == KeyModifiers::NONE => vec![Action::Quit],
        KeyCode::Char('c') if key.modifiers == KeyModifiers::CONTROL => vec![Action::Quit],

        // Page switching: Tab / Shift+Tab
        KeyCode::Tab if key.modifiers == KeyModifiers::NONE => vec![Action::NextPage],
        KeyCode::Tab if key.modifiers == KeyModifiers::SHIFT => vec![Action::PrevPage],
        KeyCode::BackTab => vec![Action::PrevPage],

        // Help
        KeyCode::Char('?') => vec![Action::ToggleHelp],

        // Force refresh
        KeyCode::Char('r') => vec![Action::RefreshData],

        // Theme cycle
        KeyCode::Char('T') => vec![Action::CycleTheme],

        _ => vec![],
    }
}
