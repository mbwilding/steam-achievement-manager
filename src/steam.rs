pub fn run(id: u32, name: &str, clear: bool) {
    let (client, _single) = match steamworks::Client::init_app(id) {
        Ok(x) => x,
        Err(_) => {
            println!("({}) {} | App not in your library", id, name);
            return;
        }
    };

    let user_stats = client.user_stats();

    match user_stats.get_num_achievements() {
        Ok(x) => x,
        Err(_) => {
            println!("({}) {} | No achievements were found", id, name);
            return;
        }
    };

    let achievement_names = match user_stats.get_achievement_names() {
        Some(x) => x,
        None => {
            println!("({}) {} | Failed to get achievement names", id, name);
            return;
        }
    };

    achievement_names.iter().for_each(|x| {
        let achievement = user_stats.achievement(x);
        let _ = if clear {
            achievement.clear()
        } else {
            achievement.set()
        };
    });

    if user_stats.store_stats().is_ok() {
        println!("({}) {} | Processed", id, name);
    }
}
