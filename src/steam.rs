pub fn run(id: u32, clear: bool) -> bool {
    println!("{}: Processing", id);

    let mut failed = false;

    let client = match steamworks::Client::init_app(id) {
        Ok(x) => x,
        Err(_) => {
            eprintln!("{}: App not in your library", id);
            return exit(id, false);
        }
    };

    let user_stats = client.user_stats();

    match user_stats.get_num_achievements() {
        Ok(x) => {
            println!("{}: {} achievements found", id, x);
            x
        }
        Err(_) => {
            eprintln!("{}: No achievements were found", id);
            return exit(id, false);
        }
    };

    let achievement_names = match user_stats.get_achievement_names() {
        Some(x) => x,
        None => {
            eprintln!("{}: Failed to get achievement names", id);
            return exit(id, false);
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
            (true, true) => "UNSET",
            (false, true) => "SET",
            (_, false) => "FAIL",
        };

        if success {
            println!("{}: {} | {}", id, name, status);
        } else {
            eprintln!("{}: {} | {}", id, name, status);
            failed = true;
        }
    });

    let stored = user_stats.store_stats().is_ok();

    exit(id, stored && !failed)
}

fn exit(id: u32, success: bool) -> bool {
    if success {
        println!("{}: Success", id);
    } else {
        eprintln!("{}: Fail", id);
    }

    success
}
