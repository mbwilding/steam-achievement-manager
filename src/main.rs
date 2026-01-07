mod args;
mod steam;
mod tui;

fn main() {
    let args = args::get();

    let id = match args.id {
        Some(id) => id,
        None => match tui::prompt_for_app_id() {
            Ok(Some(id)) => id,
            Ok(None) => {
                std::process::exit(0);
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        },
    };

    let achievements = match steam::get_achievements(id) {
        Ok(achievements) => achievements,
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };

    if let Err(e) = tui::run_tui(achievements, id) {
        eprintln!("TUI error: {}", e);
        std::process::exit(1);
    }
}
