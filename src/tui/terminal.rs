use anyhow::Result;
use crossterm::{
    cursor::Show,
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::{
    io, panic,
    sync::atomic::{AtomicBool, Ordering},
};

type RawTerminal = Terminal<CrosstermBackend<io::Stdout>>;

static ACTIVE: AtomicBool = AtomicBool::new(false);

pub struct TerminalGuard {
    terminal: RawTerminal,
}

fn restore() {
    let _ = disable_raw_mode();
    let _ = execute!(
        io::stdout(),
        LeaveAlternateScreen,
        DisableMouseCapture,
        Show
    );
}

impl TerminalGuard {
    fn new() -> Result<Self> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;

        ACTIVE.store(true, Ordering::SeqCst);
        let hook = panic::take_hook();
        panic::set_hook(Box::new(move |info| {
            if ACTIVE.swap(false, Ordering::SeqCst) {
                restore();
            }
            hook(info);
        }));

        Ok(Self { terminal })
    }

    pub fn inner_mut(&mut self) -> &mut RawTerminal {
        &mut self.terminal
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        if ACTIVE.swap(false, Ordering::SeqCst) {
            restore();
        }
    }
}

pub fn run(initial_app_id: Option<u32>) -> Result<()> {
    let mut terminal = TerminalGuard::new()?;
    super::ui::run(terminal.inner_mut(), initial_app_id)?;
    Ok(())
}
