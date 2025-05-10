mod args;

fn main() {
    let args = args::get();

    let (client, _single) = match steamworks::Client::init_app(args.id) {
        Ok(x) => x,
        Err(_) => {
            println!("({}) {} | App not in your library", args.id, args.name);
            return;
        }
    };

    let user_stats = client.user_stats();

    match user_stats.get_num_achievements() {
        Ok(x) => x,
        Err(_) => {
            println!("({}) {} | No achievements were found", args.id, args.name);
            return;
        }
    };

    let achievement_names = match user_stats.get_achievement_names() {
        Some(x) => x,
        None => {
            println!(
                "({}) {} | Failed to get achievement names",
                args.id, args.name
            );
            return;
        }
    };

    achievement_names.iter().for_each(|x| {
        let achievement = user_stats.achievement(x);
        let _ = if args.clear {
            achievement.clear()
        } else {
            achievement.set()
        };
    });

    if user_stats.store_stats().is_ok() {
        println!("({}) {} | Processed", args.id, args.name);
    }
}
