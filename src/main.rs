mod args;
mod contracts;
mod helpers;
mod steam;

use futures::stream::{self, StreamExt};
use helpers::{get_app_list_all_hash, get_app_list_all_vec, get_app_list_library};
use steam::run;

#[tokio::main]
async fn main() {
    let args = args::get();

    match args.id {
        Some(id) => {
            let name = match args.name {
                Some(x) => x,
                None => get_app_list_all_hash()
                    .await
                    .get(&id)
                    .cloned()
                    .unwrap_or("Unknown".to_string()),
            };

            run(id, &name, args.clear);
        }
        None => {
            let apps_library = if args.all {
                get_app_list_all_vec().await
            } else if args.owned {
                get_app_list_library().await
            } else {
                if !args.worker {
                    println!(
                        "To see all the options, run with --help\n\
                         Make sure Steam is running and logged in"
                    );
                }
                std::process::exit(0);
            };

            let exe = std::env::current_exe().expect("Cannot get current executable name");
            _ = stream::iter(apps_library)
                .map(|app| {
                    let exe = exe.clone();
                    async move {
                        let mut cmd = tokio::process::Command::new(exe);

                        cmd.args(["--id", &app.id.to_string()]);
                        cmd.args(["--name", &app.name]);
                        if args.clear {
                            cmd.arg("--clear");
                        }
                        cmd.arg("--worker");

                        cmd.status()
                            .await
                            .expect("Failed to execute self externally");
                    }
                })
                .buffer_unordered(args.parallel)
                .collect::<Vec<_>>()
                .await;
        }
    }
}
