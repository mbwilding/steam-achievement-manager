use super::app::App;
use super::models::{AchievementStatus, SortColumn, SortOrder, Status};
use crate::steam::{self};
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

pub fn run_achievement_manager<B: Backend>(
    terminal: &mut Terminal<B>,
    initial_app_id: Option<u32>,
) -> Result<()>
where
    <B as Backend>::Error: Send + Sync + 'static,
{
    let mut app_opt = initial_app_id.and_then(|id| {
        steam::get_achievements(id)
            .ok()
            .map(|achievements| App::new(achievements, id))
    });

    let mut app_id_input = String::new();
    let mut status: Option<Status> = None;
    let mut editing_app_id = app_opt.is_none();

    loop {
        terminal.draw(|f| {
            ui(
                f,
                app_opt.as_mut(),
                &app_id_input,
                status.as_ref(),
                editing_app_id,
            )
        })?;

        if let Event::Key(key) = event::read()?
            && key.kind == KeyEventKind::Press
        {
            if editing_app_id {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => {
                        if app_opt.is_none() {
                            return Ok(());
                        } else {
                            editing_app_id = false;
                            app_id_input.clear();
                            status = None;
                        }
                    }
                    KeyCode::Char('c') | KeyCode::Char('d') => {
                        app_id_input.clear();
                        status = None;
                    }
                    KeyCode::Char(c) if c.is_ascii_digit() => {
                        if app_id_input.len() < 10 {
                            app_id_input.push(c);
                            status = None;
                        }
                    }
                    KeyCode::Backspace => {
                        app_id_input.pop();
                        status = None;
                    }
                    KeyCode::Enter => {
                        if app_id_input.is_empty() {
                            if app_opt.is_none() {
                                return Ok(());
                            } else {
                                editing_app_id = false;
                                app_id_input.clear();
                                status = None;
                            }
                        } else {
                            match app_id_input.parse::<u32>() {
                                Ok(id) => match steam::get_achievements(id) {
                                    Ok(achievements) => {
                                        app_opt = Some(App::new(achievements, id));
                                        editing_app_id = false;
                                        app_id_input.clear();
                                        status = None;
                                    }
                                    Err(e) => {
                                        status = Some(Status::error(e.to_string()));
                                        app_id_input.clear();
                                    }
                                },
                                Err(_) => {
                                    status = Some(Status::error(format!(
                                        "Invalid App ID: {}",
                                        app_id_input
                                    )));
                                    app_id_input.clear();
                                }
                            }
                        }
                    }
                    _ => {}
                }
            } else if let Some(app) = app_opt.as_mut() {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => {
                        return Ok(());
                    }
                    KeyCode::Char('i') => {
                        editing_app_id = true;
                        app_id_input.clear();
                        status = None;
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
}

fn ui(
    f: &mut Frame,
    app: Option<&mut App>,
    app_id_input: &str,
    status: Option<&Status>,
    editing_app_id: bool,
) {
    render(f, app, app_id_input, status, editing_app_id);
}

fn render(
    f: &mut Frame,
    mut app: Option<&mut App>,
    app_id_input: &str,
    status: Option<&Status>,
    editing_app_id: bool,
) {
    let help_items = if editing_app_id {
        vec![
            ("0-9", "Type"),
            ("c/d", "Clear"),
            ("Backspace", "Delete"),
            ("Enter", "Load"),
            ("q/Esc", "Cancel"),
        ]
    } else {
        vec![
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
            ("i", "Switch App"),
            ("q/Esc", "Quit"),
        ]
    };

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

    let header = Paragraph::new(if editing_app_id || app.is_none() {
        format!("App ID: {}", app_id_input)
    } else {
        format!(
            "Steam Achievement Manager - App ID: {}",
            app.as_ref().unwrap().app_id
        )
    })
    .style(
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    )
    .block(Block::default().borders(Borders::ALL));
    f.render_widget(header, chunks[0]);

    if let Some(ref mut app) = app {
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
        let achievements_percentage =
            (achievements_done as f64 / achievements_total as f64) * 100.0;
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
    } else {
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

        let table = Table::new(
            Vec::<Row>::new(),
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
                .title(" Achievements "),
        );
        f.render_widget(table, chunks[1]);
    }

    let editing_status_holder;
    let (status_text, status_style) = if editing_app_id || app.is_none() {
        if let Some(status) = status {
            (status.message.as_str(), status.style())
        } else {
            editing_status_holder = Status::info(
                "Editing App ID - Type the app ID and press Enter to load".to_string(),
            );
            (
                editing_status_holder.message.as_str(),
                editing_status_holder.style(),
            )
        }
    } else if let Some(ref app) = app {
        if let Some(ref status) = app.status {
            (status.message.as_str(), status.style())
        } else {
            ("", Style::default())
        }
    } else {
        ("", Style::default())
    };

    let status_para = Paragraph::new(status_text)
        .style(status_style)
        .block(Block::default().borders(Borders::ALL).title(" Status "));
    f.render_widget(status_para, chunks[2]);

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
