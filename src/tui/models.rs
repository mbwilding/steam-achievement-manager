use ratatui::style::{Color, Style};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, PartialEq, Debug)]
pub enum AchievementStatus {
    Unchanged,
    Success,
    Failed,
}

#[derive(Clone, Debug)]
pub enum StatusLevel {
    Info,
    Success,
    Error,
}

#[derive(Clone, Debug)]
pub struct Status {
    pub message: String,
    pub level: StatusLevel,
}

impl Status {
    pub fn new(message: String, level: StatusLevel) -> Self {
        Self { message, level }
    }

    pub fn error(message: String) -> Self {
        Self::new(message, StatusLevel::Error)
    }

    pub fn success(message: String) -> Self {
        Self::new(message, StatusLevel::Success)
    }

    pub fn info(message: String) -> Self {
        Self::new(message, StatusLevel::Info)
    }

    pub fn style(&self) -> Style {
        match self.level {
            StatusLevel::Info => Style::default().fg(Color::Yellow),
            StatusLevel::Success => Style::default().fg(Color::Green),
            StatusLevel::Error => Style::default().fg(Color::Red),
        }
    }
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum SortColumn {
    Percentage,
    Name,
}

impl fmt::Display for SortColumn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SortColumn::Percentage => write!(f, "Percentage"),
            SortColumn::Name => write!(f, "Name"),
        }
    }
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum SortOrder {
    Ascending,
    Descending,
}

impl fmt::Display for SortOrder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SortOrder::Ascending => write!(f, "Ascending"),
            SortOrder::Descending => write!(f, "Descending"),
        }
    }
}

#[derive(Clone, Debug)]
pub struct AchievementItem {
    pub name: String,
    pub selected: bool,
    pub unlocked: bool,
    pub percentage: f32,
    pub status: AchievementStatus,
}
