mod args;
mod steam;
mod tui;

fn main() {
    let args = args::get();

    if let Err(e) = tui::run(args.id) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
