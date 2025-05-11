mod args;
mod contracts;
mod helpers;
mod steam;

use futures::stream::{self, StreamExt};
use steam::run;

#[tokio::main]
async fn main() {
    let mut args = args::get();

    if !args.worker {
        println!("Make sure Steam is running and logged in");
        println!("Otherwise the following will all fail");
    }

    match args.id {
        Some(id) => {
            if args.name.is_none() {
                let apps_all = helpers::get_app_list_all().await;
                args.name = apps_all.get(&id).cloned();
            }
            run(args);
        }
        None => {
            let apps_library = helpers::get_app_list_library().await;

            if args.parallel == 1 {
                for app in &apps_library {
                    let mut new_args = args.clone();
                    new_args.id = Some(app.id);
                    new_args.name = Some(app.name.clone());
                    run(new_args);
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

                            cmd.status().await.expect("Failed to execute sau_worker");
                        }
                    })
                    .buffer_unordered(args.parallel)
                    .collect::<Vec<_>>()
                    .await;
            }
        }
    }
}
