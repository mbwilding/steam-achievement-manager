use crate::steam::{AchievementData, get_achievements, process_achievements};
use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Frame, Terminal,
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState, Wrap},
};
use std::io;

const COLOR_LEGENDARY: Color = Color::Rgb(255, 128, 0);
const BOUND_LEGENDARY: f32 = 1.0;

const COLOR_EPIC: Color = Color::Rgb(163, 53, 238);
const BOUND_EPIC: f32 = 10.0;

const COLOR_RARE: Color = Color::Rgb(0, 112, 221);
const BOUND_RARE: f32 = 25.0;

const COLOR_UNCOMMON: Color = Color::Rgb(30, 255, 0);
const BOUND_UNCOMMON: f32 = 50.0;

const COLOR_COMMON: Color = Color::Rgb(255, 255, 255);

pub fn prompt_for_app_id() -> Result<Option<u32>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut input = String::new();
    let mut error_message = String::new();

    let result = loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Percentage(35),
                    Constraint::Length(3),
                    Constraint::Length(5),
                    Constraint::Length(3),
                    Constraint::Percentage(35),
                ])
                .split(f.area());

            let title = Paragraph::new("Steam Achievement Manager")
                .style(
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )
                .alignment(Alignment::Center)
                .block(Block::default().borders(Borders::ALL));
            f.render_widget(title, chunks[1]);

            let prompt = Paragraph::new(vec![
                Line::from(format!("Enter Steam App ID: {}", input)),
                Line::from(""),
                Line::from(vec![
                    Span::styled("Enter", Style::default().fg(Color::Yellow)),
                    Span::raw(" Confirm  "),
                    Span::styled("q/Esc", Style::default().fg(Color::Yellow)),
                    Span::raw(" Cancel"),
                ]),
            ])
            .block(Block::default().borders(Borders::ALL))
            .wrap(Wrap { trim: false });
            f.render_widget(prompt, chunks[2]);

            if !error_message.is_empty() {
                let error = Paragraph::new(error_message.as_str())
                    .style(Style::default().fg(Color::Red))
                    .alignment(Alignment::Center)
                    .block(Block::default().borders(Borders::ALL));
                f.render_widget(error, chunks[3]);
            }
        })?;

        if let Event::Key(key) = event::read()?
            && key.kind == KeyEventKind::Press
        {
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => {
                    break Ok(None);
                }
                KeyCode::Char(c) if c.is_ascii_digit() => {
                    input.push(c);
                    error_message.clear();
                }
                KeyCode::Backspace => {
                    input.pop();
                    error_message.clear();
                }
                KeyCode::Enter => match input.parse::<u32>() {
                    Ok(id) => match get_achievements(id) {
                        Ok(_) => {
                            break Ok(Some(id));
                        }
                        Err(e) => {
                            error_message = e.to_string();
                            input.clear();
                        }
                    },
                    Err(_) => {
                        error_message = format!("App {} not in your library", input);
                        input.clear();
                    }
                },
                _ => {}
            }
        }
    };

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

#[derive(Clone, PartialEq)]
pub enum AchievementStatus {
    Unchanged,
    Success,
    Failed,
}

#[derive(Clone)]
pub struct AchievementItem {
    pub name: String,
    pub selected: bool,
    pub unlocked: bool,
    pub percentage: f32,
    pub status: AchievementStatus,
}

pub struct App {
    pub achievements: Vec<AchievementItem>,
    pub current_index: usize,
    pub app_id: u32,
    pub table_state: TableState,
    pub status_message: String,
}

impl App {
    pub fn new(achievements: AchievementData, app_id: u32) -> Self {
        let mut achievements: Vec<AchievementItem> = achievements
            .achievements
            .into_iter()
            .map(|info| AchievementItem {
                name: info.name,
                selected: info.unlocked,
                unlocked: info.unlocked,
                percentage: info.percentage,
                status: AchievementStatus::Unchanged,
            })
            .collect();

        achievements.sort_by(|a, b| {
            match b.percentage.partial_cmp(&a.percentage) {
                Some(ordering) => ordering,
                None => std::cmp::Ordering::Equal,
            }
        });

        let mut table_state = TableState::default();
        table_state.select(Some(0));

        Self {
            achievements,
            current_index: 0,
            app_id,
            table_state,
            status_message: String::new(),
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
            self.table_state.select(Some(self.current_index));
        }
    }

    fn previous(&mut self) {
        if !self.achievements.is_empty() {
            if self.current_index > 0 {
                self.current_index -= 1;
            } else {
                self.current_index = self.achievements.len() - 1;
            }
            self.table_state.select(Some(self.current_index));
        }
    }

    fn process_changes(&mut self) {
        let to_set: Vec<String> = self
            .achievements
            .iter()
            .filter(|a| a.selected && !a.unlocked)
            .map(|a| a.name.clone())
            .collect();

        let to_clear: Vec<String> = self
            .achievements
            .iter()
            .filter(|a| !a.selected && a.unlocked)
            .map(|a| a.name.clone())
            .collect();

        let mut success_count = 0;
        let mut fail_count = 0;

        if !to_set.is_empty() {
            match process_achievements(self.app_id, to_set.clone(), false) {
                Ok(results) => {
                    for result in results {
                        if let Some(achievement) =
                            self.achievements.iter_mut().find(|a| a.name == result.name)
                        {
                            if result.success {
                                achievement.status = AchievementStatus::Success;
                                achievement.unlocked = true;
                                success_count += 1;
                            } else {
                                achievement.status = AchievementStatus::Failed;
                                fail_count += 1;
                            }
                        }
                    }
                }
                Err(e) => {
                    self.status_message = format!("Error: {}", e);
                    for name in to_set {
                        if let Some(achievement) =
                            self.achievements.iter_mut().find(|a| a.name == name)
                        {
                            achievement.status = AchievementStatus::Failed;
                            fail_count += 1;
                        }
                    }
                }
            }
        }

        if !to_clear.is_empty() {
            match process_achievements(self.app_id, to_clear.clone(), true) {
                Ok(results) => {
                    for result in results {
                        if let Some(achievement) =
                            self.achievements.iter_mut().find(|a| a.name == result.name)
                        {
                            if result.success {
                                achievement.status = AchievementStatus::Success;
                                achievement.unlocked = false;
                                success_count += 1;
                            } else {
                                achievement.status = AchievementStatus::Failed;
                                fail_count += 1;
                            }
                        }
                    }
                }
                Err(e) => {
                    self.status_message = format!("Error: {}", e);
                    for name in to_clear {
                        if let Some(achievement) =
                            self.achievements.iter_mut().find(|a| a.name == name)
                        {
                            achievement.status = AchievementStatus::Failed;
                            fail_count += 1;
                        }
                    }
                }
            }
        }

        if fail_count == 0 && success_count > 0 {
            self.status_message =
                format!("✓ Successfully processed {} achievement(s)", success_count);
        } else if fail_count > 0 {
            self.status_message = format!(
                "⚠ Processed: {} success, {} failed",
                success_count, fail_count
            );
        }
    }
}

pub fn run_tui(achievements: AchievementData, app_id: u32) -> Result<Option<Vec<(String, bool)>>> {
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
) -> Result<Option<Vec<(String, bool)>>> {
    loop {
        terminal.draw(|f| ui(f, &mut app)).unwrap();

        if let Event::Key(key) = event::read()?
            && key.kind == KeyEventKind::Press
        {
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
                KeyCode::Enter => {
                    app.process_changes();
                }
                _ => {}
            }
        }
    }
}

fn ui(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
            Constraint::Length(3),
        ])
        .split(f.area());

    // Header
    let header = Paragraph::new(format!(
        "Steam Achievement Manager - App ID: {}",
        app.app_id
    ))
    .style(
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    )
    .block(Block::default().borders(Borders::ALL));
    f.render_widget(header, chunks[0]);

    // Achievement table
    let header = Row::new(vec![
        Cell::from("Done").style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Cell::from("Global").style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Cell::from("Achievement Name").style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
    ])
    .height(1);

    let rows: Vec<Row> = app
        .achievements
        .iter()
        .map(|achievement| {
            let checkbox = if achievement.selected { "[✓]" } else { "[ ]" };

            let percentage_style = if achievement.percentage <= BOUND_LEGENDARY {
                Style::default()
                    .fg(COLOR_LEGENDARY)
                    .add_modifier(Modifier::BOLD)
            } else if achievement.percentage <= BOUND_EPIC {
                Style::default().fg(COLOR_EPIC).add_modifier(Modifier::BOLD)
            } else if achievement.percentage <= BOUND_RARE {
                Style::default().fg(COLOR_RARE)
            } else if achievement.percentage <= BOUND_UNCOMMON {
                Style::default().fg(COLOR_UNCOMMON)
            } else {
                Style::default().fg(COLOR_COMMON)
            };

            let checkbox_style = match achievement.status {
                AchievementStatus::Failed => Style::default().fg(Color::Red),
                AchievementStatus::Success => Style::default().fg(Color::Green),
                AchievementStatus::Unchanged => {
                    if achievement.selected {
                        Style::default().fg(Color::Green)
                    } else {
                        Style::default()
                    }
                }
            };

            let name_style = match achievement.status {
                AchievementStatus::Failed => Style::default().fg(Color::Red),
                AchievementStatus::Success => Style::default().fg(Color::Green),
                AchievementStatus::Unchanged => Style::default(),
            };

            Row::new(vec![
                Cell::from(checkbox).style(checkbox_style),
                Cell::from(format!("{:.1}%", achievement.percentage)).style(percentage_style),
                Cell::from(achievement.name.clone()).style(name_style),
            ])
        })
        .collect();

    let achievements_done = app.achievements.iter().filter(|x| x.unlocked).count();
    let achievements_total = app.achievements.len();
    let achievements_percentage = (achievements_done as f64 / achievements_total as f64) * 100.0;
    let achievements_style = if achievements_percentage == 100.0 {
        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
    } else if achievements_percentage > 90.0 {
        Style::default()
            .fg(COLOR_LEGENDARY)
            .add_modifier(Modifier::BOLD)
    } else if achievements_percentage > 75.0 {
        Style::default().fg(COLOR_EPIC).add_modifier(Modifier::BOLD)
    } else if achievements_percentage > 50.0 {
        Style::default().fg(COLOR_RARE)
    } else if achievements_percentage > 25.0 {
        Style::default().fg(COLOR_UNCOMMON)
    } else {
        Style::default().fg(COLOR_COMMON)
    };

    let table = Table::new(
        rows,
        [
            Constraint::Length(4), // Done
            Constraint::Length(6), // Global
            Constraint::Fill(1),   // Achievement Name
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(Line::from(vec![
                Span::styled(
                    " Achievements ",
                    Style::default()
                        .fg(Color::Magenta)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("{}/{} ", achievements_done, achievements_total),
                    achievements_style,
                ),
            ])),
    )
    .row_highlight_style(
        Style::default()
            .bg(Color::Rgb(0x18, 0x18, 0x18))
            .add_modifier(Modifier::BOLD),
    );
    f.render_stateful_widget(table, chunks[1], &mut app.table_state);

    // Status message
    let status_style = if app.status_message.contains("failed") {
        Style::default().fg(Color::Red)
    } else if !app.status_message.is_empty() {
        Style::default().fg(Color::Green)
    } else {
        Style::default()
    };

    let status = Paragraph::new(app.status_message.as_str())
        .style(status_style)
        .block(Block::default().borders(Borders::ALL).title(" Status "));
    f.render_widget(status, chunks[2]);

    // Help text
    let help_text = vec![Line::from(vec![
        Span::styled("↑/k", Style::default().fg(Color::Yellow)),
        Span::raw(" Up  "),
        Span::styled("↓/j", Style::default().fg(Color::Yellow)),
        Span::raw(" Down  "),
        Span::styled("Space", Style::default().fg(Color::Yellow)),
        Span::raw(" Toggle  "),
        Span::styled("a", Style::default().fg(Color::Yellow)),
        Span::raw(" Select All  "),
        Span::styled("d", Style::default().fg(Color::Yellow)),
        Span::raw(" Deselect All  "),
        Span::styled("Enter", Style::default().fg(Color::Yellow)),
        Span::raw(" Process  "),
        Span::styled("q/Esc", Style::default().fg(Color::Yellow)),
        Span::raw(" Quit"),
    ])];

    let help = Paragraph::new(help_text).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Controls ")
            .style(Style::default().fg(Color::Gray)),
    );
    f.render_widget(help, chunks[3]);
}
