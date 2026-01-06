mod args;
mod steam;

use steam::run;

#[tokio::main]
async fn main() {
    let args = args::get();

    match args.id {
        Some(id) => {
            run(id, args.clear);
        }
        None => {
            if !args.worker {
                println!(
                    "To see all the options, run with --help\n\
                     Make sure Steam is running and logged in"
                );
            }
            std::process::exit(0);
        }
    }
}
