use std::fmt;

#[derive(Clone, PartialEq)]
pub enum AchievementStatus {
    Unchanged,
    Success,
    Failed,
}

#[derive(Clone, PartialEq)]
pub enum SortColumn {
    Percentage,
    Name,
}

impl SortColumn {
    pub fn from_string(s: &str) -> Self {
        match s {
            "Name" => SortColumn::Name,
            _ => SortColumn::Percentage,
        }
    }
}

impl fmt::Display for SortColumn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SortColumn::Percentage => write!(f, "Percentage"),
            SortColumn::Name => write!(f, "Name"),
        }
    }
}

#[derive(Clone, PartialEq)]
pub enum SortOrder {
    Ascending,
    Descending,
}

impl SortOrder {
    pub fn from_string(s: &str) -> Self {
        match s {
            "Ascending" => SortOrder::Ascending,
            _ => SortOrder::Descending,
        }
    }
}

impl fmt::Display for SortOrder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SortOrder::Ascending => write!(f, "Ascending"),
            SortOrder::Descending => write!(f, "Descending"),
        }
    }
}

#[derive(Clone)]
pub struct AchievementItem {
    pub name: String,
    pub selected: bool,
    pub unlocked: bool,
    pub percentage: f32,
    pub status: AchievementStatus,
}
