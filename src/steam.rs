use colored::{ColoredString, Colorize};

pub fn run(id: u32, clear: bool) -> bool {
    let id_colored = id.to_string().blue();

    println!("{}: {}", id_colored, "Processing".green());

    let mut failed = false;

    let client = match steamworks::Client::init_app(id) {
        Ok(x) => x,
        Err(_) => {
            eprintln!("{}: App not in your library", id_colored);
            return exit(&id_colored, false);
        }
    };

    let user_stats = client.user_stats();

    match user_stats.get_num_achievements() {
        Ok(count) => {
            println!(
                "{}: {} | {}",
                id_colored,
                "Achievements".green(),
                count.to_string().bright_blue(),
            );
            count
        }
        Err(_) => {
            eprintln!("{}: No achievements were found", id_colored);
            return exit(&id_colored, false);
        }
    };

    let achievement_names = match user_stats.get_achievement_names() {
        Some(x) => x,
        None => {
            eprintln!("{}: Failed to get achievement names", id_colored);
            return exit(&id_colored, false);
        }
    };

    achievement_names.iter().for_each(|name| {
        let achievement = user_stats.achievement(name);

        let success = if clear {
            achievement.clear()
        } else {
            achievement.set()
        }
        .is_ok();

        let status = match (clear, success) {
            (true, true) => "UNSET".yellow(),
            (false, true) => "SET".green(),
            (_, false) => "FAIL".red(),
        };

        let name_colored = name.bright_blue();
        if success {
            println!("{}: {} | {}", id_colored, status, name_colored);
        } else {
            eprintln!("{}: {} | {}", id_colored, status, name_colored);
            failed = true;
        }
    });

    let stored = user_stats.store_stats().is_ok();

    exit(&id_colored, stored && !failed)
}

fn exit(id_colored: &ColoredString, success: bool) -> bool {
    if success {
        println!("{}: {}", id_colored, "Success".green());
    } else {
        eprintln!("{}: {}", id_colored, "Failed".red());
    }

    success
}
