mod args;
mod contracts;
mod helpers;

use futures::stream::{self, StreamExt};

#[tokio::main]
async fn main() {
    let args = args::get_and_validate();

    println!("Make sure Steam is running and logged in");
    println!("Otherwise the following will all fail");

    let apps = helpers::get_app_list().await;

    let mut worker = std::env::current_exe().expect("Cannot get current executable name");
    worker.set_file_name(format!("SAU-Worker{}", std::env::consts::EXE_SUFFIX));

    match args.id {
        Some(id) => {
            let mut cmd = tokio::process::Command::new(worker);

            cmd.args(["--id", &id.to_string()]);
            if let Some(name) = args.name {
                cmd.args(["--name", &name]);
            }
            if args.clear {
                cmd.arg("--clear");
            }

            cmd.status().await.expect("failed to execute sau_worker");
        }
        None => {
            _ = stream::iter(apps)
                .map(|app| {
                    let worker = worker.clone();
                    async move {
                        let mut cmd = tokio::process::Command::new(worker);

                        cmd.args(["--id", &app.id.to_string()]);
                        cmd.args(["--name", &app.name]);
                        if args.clear {
                            cmd.arg("--clear");
                        }

                        cmd.status().await.expect("failed to execute sau_worker");
                    }
                })
                .buffer_unordered(args.parallel)
                .collect::<Vec<_>>()
                .await;
        }
    }
}
