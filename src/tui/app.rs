use super::config::AppConfig;
use super::models::{AchievementItem, AchievementStatus, SortColumn, SortOrder, Status};
use crate::steam::{AchievementData, process_achievements};
use ratatui::widgets::TableState;

fn is_word_boundary(prev: Option<char>, current: char) -> bool {
    match prev {
        None => true,
        Some(p) => {
            (p == ' ' || p == '_' || p == '-' || p == ':' || p == '/' || p == '(' || p == '[')
                || (p.is_ascii_lowercase() && current.is_ascii_uppercase())
        }
    }
}

fn fuzzy_score(haystack: &str, needle: &str) -> Option<i64> {
    let needle = needle.trim();
    if needle.is_empty() {
        return None;
    }

    if haystack.contains(needle) {
        return Some(1_000_000 - (haystack.len() as i64 - needle.len() as i64));
    }

    let mut score: i64 = 0;
    let mut last_match_index: Option<usize> = None;

    let mut hay_chars = haystack.chars().enumerate();
    let mut prev_char: Option<char> = None;

    for needle_char in needle.chars() {
        let mut found: Option<(usize, char, Option<char>)> = None;

        for (i, h) in hay_chars.by_ref() {
            if h == needle_char {
                found = Some((i, h, prev_char));
                prev_char = Some(h);
                break;
            }
            prev_char = Some(h);
        }

        let Some((i, h, prev)) = found else {
            return None;
        };

        score += 10;

        if let Some(last) = last_match_index {
            if i == last + 1 {
                score += 15;
            } else {
                score -= (i.saturating_sub(last + 1) as i64) * 2;
            }
        }

        if is_word_boundary(prev, h) {
            score += 20;
        }

        last_match_index = Some(i);
    }

    if let Some(first) = last_match_index {
        score -= first as i64;
    }

    Some(score)
}

pub struct App {
    pub achievements: Vec<AchievementItem>,
    pub current_index: usize,
    pub app_id: u32,
    pub table_state: TableState,
    pub status: Option<Status>,
    pub sort_column: SortColumn,
    pub sort_order: SortOrder,
    pub search_query: String,
}

impl App {
    pub fn new(achievements: AchievementData, app_id: u32) -> Self {
        let config: AppConfig = confy::load("sam", None).unwrap_or_default();

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

        achievements.sort_by(|a, b| match b.percentage.partial_cmp(&a.percentage) {
            Some(ordering) => ordering,
            None => std::cmp::Ordering::Equal,
        });

        let mut table_state = TableState::default();
        table_state.select(Some(0));

        let mut app = Self {
            achievements,
            current_index: 0,
            app_id,
            table_state,
            status: None,
            sort_column: config.sort_column,
            sort_order: config.sort_order,
            search_query: String::new(),
        };

        app.sort_achievements();
        app
    }

    pub fn toggle_selection(&mut self) {
        if !self.achievements.is_empty() {
            self.achievements[self.current_index].selected =
                !self.achievements[self.current_index].selected;
        }
    }

    pub fn select_all(&mut self) {
        for achievement in &mut self.achievements {
            achievement.selected = true;
        }
    }

    pub fn deselect_all(&mut self) {
        for achievement in &mut self.achievements {
            achievement.selected = false;
        }
    }

    pub fn next(&mut self) {
        if !self.achievements.is_empty() {
            self.current_index = (self.current_index + 1) % self.achievements.len();
            self.table_state.select(Some(self.current_index));
        }
    }

    pub fn jump_to(&mut self, index: usize) {
        if !self.achievements.is_empty() {
            self.current_index = index.min(self.achievements.len() - 1);
            self.table_state.select(Some(self.current_index));
        }
    }

    pub fn search_first_match(&mut self) -> bool {
        if self.search_query.trim().is_empty() {
            return false;
        }

        let query = self.search_query.to_lowercase();

        let mut best: Option<(usize, i64)> = None;
        for (index, achievement) in self.achievements.iter().enumerate() {
            let name = achievement.name.to_lowercase();
            if let Some(score) = fuzzy_score(&name, &query) {
                match best {
                    None => best = Some((index, score)),
                    Some((_, best_score)) if score > best_score => best = Some((index, score)),
                    _ => {}
                }
            }
        }

        if let Some((index, _)) = best {
            self.jump_to(index);
            return true;
        }

        false
    }

    pub fn previous(&mut self) {
        if !self.achievements.is_empty() {
            if self.current_index > 0 {
                self.current_index -= 1;
            } else {
                self.current_index = self.achievements.len() - 1;
            }
            self.table_state.select(Some(self.current_index));
        }
    }

    pub fn jump_to_top(&mut self) {
        if !self.achievements.is_empty() {
            self.current_index = 0;
            self.table_state.select(Some(self.current_index));
        }
    }

    pub fn jump_to_bottom(&mut self) {
        if !self.achievements.is_empty() {
            self.current_index = self.achievements.len() - 1;
            self.table_state.select(Some(self.current_index));
        }
    }

    pub fn page_up(&mut self) {
        if !self.achievements.is_empty() {
            let page_size = 10;
            if self.current_index >= page_size {
                self.current_index -= page_size;
            } else {
                self.current_index = 0;
            }
            self.table_state.select(Some(self.current_index));
        }
    }

    pub fn page_down(&mut self) {
        if !self.achievements.is_empty() {
            let page_size = 10;
            let max_index = self.achievements.len() - 1;
            if self.current_index + page_size <= max_index {
                self.current_index += page_size;
            } else {
                self.current_index = max_index;
            }
            self.table_state.select(Some(self.current_index));
        }
    }

    pub fn process_changes(&mut self) {
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
                    self.status = Some(Status::error(e.to_string()));
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
                    self.status = Some(Status::error(e.to_string()));
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
            self.status = Some(Status::success(format!(
                "✓ Successfully processed {} achievement(s)",
                success_count
            )));
        } else if fail_count > 0 {
            self.status = Some(Status::error(format!(
                "⚠ Processed: {} success, {} failed",
                success_count, fail_count
            )));
        }
    }

    pub fn sort_achievements(&mut self) {
        match self.sort_column {
            SortColumn::Percentage => {
                self.achievements.sort_by(|a, b| {
                    let ordering = match a.percentage.partial_cmp(&b.percentage) {
                        Some(ord) => ord,
                        None => std::cmp::Ordering::Equal,
                    };
                    if self.sort_order == SortOrder::Ascending {
                        ordering
                    } else {
                        ordering.reverse()
                    }
                });
            }
            SortColumn::Name => {
                self.achievements.sort_by(|a, b| {
                    let ordering = a.name.cmp(&b.name);
                    if self.sort_order == SortOrder::Ascending {
                        ordering
                    } else {
                        ordering.reverse()
                    }
                });
            }
        }
    }

    pub fn set_sort_column(&mut self, column: SortColumn) {
        self.sort_column = column;
        self.sort_achievements();
        self.save_config();
    }

    pub fn toggle_sort_order(&mut self) {
        self.sort_order = if self.sort_order == SortOrder::Ascending {
            SortOrder::Descending
        } else {
            SortOrder::Ascending
        };
        self.sort_achievements();
        self.save_config();
    }

    fn save_config(&self) {
        let config = AppConfig {
            sort_column: self.sort_column.clone(),
            sort_order: self.sort_order.clone(),
        };
        let _ = confy::store("sam", None, config);
    }
}
