use super::terminal;
use crate::steam::get_achievements;
use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};

pub fn prompt_for_app_id() -> Result<Option<u32>> {
    let mut terminal = terminal::setup()?;

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

    terminal::teardown(&mut terminal)?;

    result
}
