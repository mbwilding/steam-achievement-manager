mod args;
mod steam;
mod tui;

use steam::{get_achievements, process_achievements, run};

fn main() {
    let args = args::get();

    if args.id.is_empty() {
        if !args.worker {
            println!(
                "To see all the options, run with --help\n\
                 Make sure Steam is running and logged in"
            );
        }
        std::process::exit(0);
    }

    if args.tui {
        // TUI mode - only works with single app ID
        if args.id.len() != 1 {
            eprintln!("TUI mode only supports a single app ID at a time");
            std::process::exit(1);
        }

        let id = args.id[0];
        match get_achievements(id) {
            Ok(data) => {
                match tui::run_tui(data, id) {
                    Ok(Some((selected_achievements, clear))) => {
                        // Process the selected achievements
                        process_achievements(id, selected_achievements, clear);
                    }
                    Ok(None) => {
                        println!("Operation cancelled");
                    }
                    Err(e) => {
                        eprintln!("TUI error: {}", e);
                        std::process::exit(1);
                    }
                }
            }
            Err(e) => {
                eprintln!("{}", e);
                std::process::exit(1);
            }
        }
    } else {
        // CLI mode - original behavior
        for id in args.id {
            run(id, args.clear);
        }
    }
}
