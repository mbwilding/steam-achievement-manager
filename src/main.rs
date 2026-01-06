mod args;
mod steam;

use steam::run;

#[tokio::main]
async fn main() {
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

    for id in args.id {
        run(id, args.clear);
    }
}
