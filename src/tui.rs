use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Frame, Terminal,
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};
use std::io;

use crate::steam::AchievementData;

#[derive(Clone)]
pub struct AchievementItem {
    pub name: String,
    pub selected: bool,
}

pub struct App {
    pub achievements: Vec<AchievementItem>,
    pub current_index: usize,
    pub app_id: u32,
    pub mode: SelectionMode,
}

#[derive(Clone, Copy, PartialEq)]
pub enum SelectionMode {
    Set,
    Clear,
}

impl App {
    pub fn new(achievements: AchievementData, app_id: u32) -> Self {
        let achievements = achievements.names
            .into_iter()
            .map(|name| AchievementItem {
                name,
                selected: false,
            })
            .collect();

        Self {
            achievements,
            current_index: 0,
            app_id,
            mode: SelectionMode::Set,
        }
    }

    fn toggle_selection(&mut self) {
        if !self.achievements.is_empty() {
            self.achievements[self.current_index].selected =
                !self.achievements[self.current_index].selected;
        }
    }

    fn select_all(&mut self) {
        for achievement in &mut self.achievements {
            achievement.selected = true;
        }
    }

    fn deselect_all(&mut self) {
        for achievement in &mut self.achievements {
            achievement.selected = false;
        }
    }

    fn next(&mut self) {
        if !self.achievements.is_empty() {
            self.current_index = (self.current_index + 1) % self.achievements.len();
        }
    }

    fn previous(&mut self) {
        if !self.achievements.is_empty() {
            if self.current_index > 0 {
                self.current_index -= 1;
            } else {
                self.current_index = self.achievements.len() - 1;
            }
        }
    }

    fn toggle_mode(&mut self) {
        self.mode = match self.mode {
            SelectionMode::Set => SelectionMode::Clear,
            SelectionMode::Clear => SelectionMode::Set,
        };
    }

    pub fn get_selected_achievements(&self) -> Vec<String> {
        self.achievements
            .iter()
            .filter(|a| a.selected)
            .map(|a| a.name.clone())
            .collect()
    }
}

pub fn run_tui(achievements: AchievementData, app_id: u32) -> Result<Option<(Vec<String>, bool)>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app and run
    let app = App::new(achievements, app_id);
    let result = run_app(&mut terminal, app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result
}

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
) -> Result<Option<(Vec<String>, bool)>> {
    loop {
        terminal.draw(|f| ui(f, &app)).unwrap();

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => {
                        return Ok(None);
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        app.next();
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        app.previous();
                    }
                    KeyCode::Char(' ') => {
                        app.toggle_selection();
                    }
                    KeyCode::Char('a') => {
                        app.select_all();
                    }
                    KeyCode::Char('d') => {
                        app.deselect_all();
                    }
                    KeyCode::Char('m') => {
                        app.toggle_mode();
                    }
                    KeyCode::Enter => {
                        let selected = app.get_selected_achievements();
                        if !selected.is_empty() {
                            let clear = app.mode == SelectionMode::Clear;
                            return Ok(Some((selected, clear)));
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}

fn ui(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(6),
        ])
        .split(f.area());

    // Header
    let mode_text = match app.mode {
        SelectionMode::Set => "SET",
        SelectionMode::Clear => "CLEAR",
    };
    let mode_color = match app.mode {
        SelectionMode::Set => Color::Green,
        SelectionMode::Clear => Color::Yellow,
    };

    let header = Paragraph::new(format!(
        "Steam Achievement Manager - App ID: {} | Mode: {}",
        app.app_id, mode_text
    ))
    .style(
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    )
    .block(Block::default().borders(Borders::ALL));
    f.render_widget(header, chunks[0]);

    // Achievement list
    let items: Vec<ListItem> = app
        .achievements
        .iter()
        .enumerate()
        .map(|(i, achievement)| {
            let checkbox = if achievement.selected { "[✓]" } else { "[ ]" };
            let content = format!("{} {}", checkbox, achievement.name);

            let style = if i == app.current_index {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else if achievement.selected {
                Style::default().fg(mode_color)
            } else {
                Style::default()
            };

            ListItem::new(content).style(style)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!(" Achievements ({}) ", app.achievements.len())),
    );
    f.render_widget(list, chunks[1]);

    // Help text
    let help_text = vec![
        Line::from(vec![
            Span::styled("↑/k", Style::default().fg(Color::Yellow)),
            Span::raw(" Up  "),
            Span::styled("↓/j", Style::default().fg(Color::Yellow)),
            Span::raw(" Down  "),
            Span::styled("Space", Style::default().fg(Color::Yellow)),
            Span::raw(" Toggle  "),
            Span::styled("a", Style::default().fg(Color::Yellow)),
            Span::raw(" Select All  "),
            Span::styled("d", Style::default().fg(Color::Yellow)),
            Span::raw(" Deselect All"),
        ]),
        Line::from(vec![
            Span::styled("m", Style::default().fg(Color::Yellow)),
            Span::raw(" Toggle Mode (Set/Clear)  "),
            Span::styled("Enter", Style::default().fg(Color::Yellow)),
            Span::raw(" Confirm  "),
            Span::styled("q/Esc", Style::default().fg(Color::Yellow)),
            Span::raw(" Quit"),
        ]),
    ];

    let help = Paragraph::new(help_text).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Controls ")
            .style(Style::default().fg(Color::Gray)),
    );
    f.render_widget(help, chunks[2]);
}
