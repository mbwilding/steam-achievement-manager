use super::app::App;
use super::models::{AchievementStatus, SortColumn, SortOrder};
use super::terminal;
use crate::steam::AchievementData;
use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::{
    Frame, Terminal,
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
};

const COLOR_LEGENDARY: Color = Color::Rgb(255, 128, 0);
const BOUND_LEGENDARY: f32 = 1.0;
const COLOR_EPIC: Color = Color::Rgb(163, 53, 238);
const BOUND_EPIC: f32 = 10.0;
const COLOR_RARE: Color = Color::Rgb(0, 112, 221);
const BOUND_RARE: f32 = 25.0;
const COLOR_UNCOMMON: Color = Color::Rgb(30, 255, 0);
const BOUND_UNCOMMON: f32 = 50.0;
const COLOR_COMMON: Color = Color::Rgb(255, 255, 255);

pub fn run_tui(achievements: AchievementData, app_id: u32) -> Result<Option<Vec<(String, bool)>>> {
    let mut terminal = terminal::setup()?;

    let app = App::new(achievements, app_id);
    let result = run_app(&mut terminal, app);

    terminal::teardown(&mut terminal)?;

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
                KeyCode::Char('g') => {
                    app.jump_to_top();
                }
                KeyCode::Char('G') => {
                    app.jump_to_bottom();
                }
                KeyCode::PageUp => {
                    app.page_up();
                }
                KeyCode::PageDown => {
                    app.page_down();
                }
                KeyCode::Char('p') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    app.page_up();
                }
                KeyCode::Char('n') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    app.page_down();
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
                KeyCode::Char('p') => {
                    app.set_sort_column(SortColumn::Percentage);
                }
                KeyCode::Char('n') => {
                    app.set_sort_column(SortColumn::Name);
                }
                KeyCode::Char('o') => {
                    app.toggle_sort_order();
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
    let help_items = vec![
        ("↑/k", "Up"),
        ("↓/j", "Down"),
        ("g", "Top"),
        ("G", "Bottom"),
        ("PgUp/PgDn/^P/^N", "Page"),
        ("Space", "Toggle"),
        ("a", "Enable All"),
        ("d", "Disable All"),
        ("p/n", "Sort Column"),
        ("o", "Order"),
        ("Enter", "Process"),
        ("q/Esc", "Quit"),
    ];

    const BORDER_WIDTH: usize = 2;
    const WRAP_TOLERANCE: usize = 4;
    const KEY_DESC_SEPARATOR: usize = 1;
    const ITEM_SPACING: usize = 2;

    let available_width = f.area().width.saturating_sub(BORDER_WIDTH as u16) as usize;
    let mut help_line_count = 0;
    let mut current_line_width = 0;
    let mut line_has_content = false;

    for (i, (key, desc)) in help_items.iter().enumerate() {
        let is_last = i == help_items.len() - 1;
        let item_base_width = key.len() + KEY_DESC_SEPARATOR + desc.len();
        let spacing_width = if is_last { 0 } else { ITEM_SPACING };

        if line_has_content
            && current_line_width + item_base_width > available_width.saturating_add(WRAP_TOLERANCE)
        {
            help_line_count += 1;
            current_line_width = 0;
        }

        current_line_width += item_base_width + spacing_width;
        line_has_content = true;
    }

    if line_has_content {
        help_line_count += 1;
    }

    let help_height = help_line_count.max(1) as u16 + BORDER_WIDTH as u16;

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
            Constraint::Length(help_height),
        ])
        .split(f.area());

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

    let sort_indicator = if app.sort_order == SortOrder::Ascending {
        "↑"
    } else {
        "↓"
    };

    let percentage_header = if app.sort_column == SortColumn::Percentage {
        format!("Global {}", sort_indicator)
    } else {
        "Global".to_string()
    };

    let name_header = if app.sort_column == SortColumn::Name {
        format!("Achievement Name {}", sort_indicator)
    } else {
        "Achievement Name".to_string()
    };

    let header = Row::new(vec![
        Cell::from("Done").style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Cell::from(percentage_header).style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Cell::from(name_header).style(
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
            Constraint::Length(6),
            Constraint::Length(8),
            Constraint::Fill(1),
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

    let mut help_lines: Vec<Line> = vec![];
    let mut current_line_spans: Vec<Span> = vec![];
    let mut current_line_width = 0;
    let available_width = f.area().width.saturating_sub(BORDER_WIDTH as u16) as usize;
    let line_width_threshold = available_width.saturating_add(WRAP_TOLERANCE);

    for (i, (key, desc)) in help_items.iter().enumerate() {
        let is_last = i == help_items.len() - 1;
        let item_base_width = key.len() + KEY_DESC_SEPARATOR + desc.len();
        let spacing_width = if is_last { 0 } else { ITEM_SPACING };

        if !current_line_spans.is_empty()
            && current_line_width + item_base_width > line_width_threshold
        {
            help_lines.push(Line::from(current_line_spans.clone()));
            current_line_spans.clear();
            current_line_width = 0;
        }

        current_line_spans.push(Span::styled(*key, Style::default().fg(Color::Yellow)));
        current_line_spans.push(Span::raw(if is_last {
            format!(" {}", desc)
        } else {
            format!(" {}  ", desc)
        }));
        current_line_width += item_base_width + spacing_width;
    }

    if !current_line_spans.is_empty() {
        help_lines.push(Line::from(current_line_spans));
    }

    let help = Paragraph::new(help_lines).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Controls ")
            .style(Style::default().fg(Color::Gray)),
    );
    f.render_widget(help, chunks[3]);
}
