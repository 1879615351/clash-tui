pub mod components;
pub mod layout;
pub mod pages;
pub mod theme;

use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::ExecutableCommand;
use ratatui::prelude::*;
use std::io::{stdout, IsTerminal, Stdout};

pub fn init() -> anyhow::Result<Terminal<CrosstermBackend<Stdout>>> {
    if !std::io::stdout().is_terminal() {
        anyhow::bail!("clash-tui requires a real terminal (not piped/redirected). Run from cmd or PowerShell.");
    }

    // Windows: pre-init console input mode so event::poll can read keyboard
    #[cfg(windows)]
    init_console_input();

    let mut stdout = stdout();
    crossterm::terminal::enable_raw_mode()?;
    stdout.execute(EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

/// Ensure console input handle is initialized so keyboard events work.
/// Uses raw FFI to avoid heavy dependencies.
#[cfg(windows)]
fn init_console_input() {
    extern "system" {
        fn GetStdHandle(nStdHandle: u32) -> isize;
        fn GetConsoleMode(hConsoleHandle: isize, lpMode: *mut u32) -> i32;
        fn SetConsoleMode(hConsoleHandle: isize, dwMode: u32) -> i32;
    }

    const STD_INPUT_HANDLE: u32 = 0xFFFF_FFF6u32;
    const ENABLE_VIRTUAL_TERMINAL_INPUT: u32 = 0x0200;
    const ENABLE_WINDOW_INPUT: u32 = 0x0008;

    unsafe {
        let handle = GetStdHandle(STD_INPUT_HANDLE);
        if handle == -1 || handle == 0 {
            return;
        }
        let mut mode: u32 = 0;
        // Only intervene if the console mode was never initialized.
        // If GetConsoleMode succeeds, the existing mode is fine — don't touch it.
        if GetConsoleMode(handle, &mut mode) == 0 {
            SetConsoleMode(handle, ENABLE_WINDOW_INPUT | ENABLE_VIRTUAL_TERMINAL_INPUT);
        }
    }
}

pub fn restore() -> anyhow::Result<()> {
    let mut stdout = stdout();
    terminal::disable_raw_mode()?;
    stdout.execute(LeaveAlternateScreen)?;
    Ok(())
}
