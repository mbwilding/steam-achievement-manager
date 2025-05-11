mod args;
mod contracts;
mod helpers;
mod steam;

use futures::stream::{self, StreamExt};
use helpers::get_app_list_all;
use steam::run;

#[tokio::main]
async fn main() {
    let args = args::get();

    if !args.worker {
        println!("Make sure Steam is running and logged in");
        println!("Otherwise the following will all fail");
    }

    match args.id {
        Some(id) => {
            let name = match args.name {
                Some(x) => x,
                None => {
                    let apps = get_app_list_all().await;
                    apps.get(&id).cloned().expect("App ID does not exist")
                }
            };

            run(id, &name, args.clear);
        }
        None => {
            let apps_library = helpers::get_app_list_library().await;

            if args.parallel == 1 {
                for app in &apps_library {
                    run(app.id, &app.name, args.clear);
                }
            } else {
                let worker = std::env::current_exe().expect("Cannot get current executable name");
                _ = stream::iter(apps_library)
                    .map(|app| {
                        let worker = worker.clone();
                        async move {
                            let mut cmd = tokio::process::Command::new(worker);

                            cmd.args(["--id", &app.id.to_string()]);
                            cmd.args(["--name", &app.name]);
                            if args.clear {
                                cmd.arg("--clear");
                            }
                            cmd.arg("--worker");

                            cmd.status().await.expect("Failed to execute self externally");
                        }
                    })
                    .buffer_unordered(args.parallel)
                    .collect::<Vec<_>>()
                    .await;
            }
        }
    }
}
