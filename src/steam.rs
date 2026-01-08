use anyhow::{Context, Result, bail};
use gag::Gag;
use steamworks::{GameId, SteamError};

pub struct AchievementInfo {
    pub name: String,
    pub unlocked: bool,
    pub percentage: f32,
}

pub struct AchievementData {
    pub achievements: Vec<AchievementInfo>,
}

pub struct ProcessResult {
    pub name: String,
    pub success: bool,
}

pub fn get_achievements(id: u32) -> Result<AchievementData> {
    let _stdout_gag = Gag::stdout().ok();
    let _stderr_gag = Gag::stderr().ok();

    let client = steamworks::Client::init_app(id)
        .with_context(|| format!("App {} not in your library", id))?;

    let user_stats = client.user_stats();

    match user_stats.get_num_achievements() {
        Ok(_) => {}
        Err(_) => bail!("Failed to get achievement names for app {}", id),
    };

    // NOTE: Required to get the global percentages
    let game_id = GameId::from_raw(id as u64);
    user_stats.request_global_achievement_percentages(move |result: Result<GameId, SteamError>| {
        result.unwrap_or(game_id);
    });

    let achievement_names = match user_stats.get_achievement_names() {
        Some(x) => x,
        None => bail!("Failed to get achievement names for app {}", id),
    };

    let achievements = achievement_names
        .into_iter()
        .map(|name| {
            let achievement = user_stats.achievement(&name);

            let unlocked = achievement.get().unwrap_or(false);

            let percentage = achievement
                .get_achievement_achieved_percent()
                .unwrap_or_default();

            AchievementInfo {
                name,
                unlocked,
                percentage,
            }
        })
        .collect();

    Ok(AchievementData { achievements })
}

pub fn process_achievements(
    id: u32,
    achievement_names: Vec<String>,
    clear: bool,
) -> Result<Vec<ProcessResult>, String> {
    let _stdout_gag = Gag::stdout().ok();
    let _stderr_gag = Gag::stderr().ok();

    let client = match steamworks::Client::init_app(id) {
        Ok(x) => x,
        Err(_) => {
            return Err(format!("App {} not in your library", id));
        }
    };

    let user_stats = client.user_stats();

    let results: Vec<ProcessResult> = achievement_names
        .iter()
        .map(|name| {
            let achievement = user_stats.achievement(name);

            let success = if clear {
                achievement.clear()
            } else {
                achievement.set()
            }
            .is_ok();

            ProcessResult {
                name: name.clone(),
                success,
            }
        })
        .collect();

    let all_success = results.iter().all(|r| r.success);
    let stored = user_stats.store_stats().is_ok();

    if all_success && stored {
        Ok(results)
    } else if !stored {
        Err("Failed to store stats to Steam".to_string())
    } else {
        Ok(results)
    }
}
