use super::terminal;
use anyhow::Result;

pub fn run(initial_app_id: Option<u32>) -> Result<()> {
    let mut terminal = terminal::setup()?;

    super::ui::run_achievement_manager(&mut terminal, initial_app_id)?;

    terminal::teardown(&mut terminal)?;
    Ok(())
}
