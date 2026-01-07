use clap::Parser;

pub fn get() -> Args {
    Args::parse()
}

#[derive(Parser, Debug, Clone)]
#[command(
    version,
    about,
    long_about = None,
    after_help = "Examples:\n  \
                  sau                               # Launch TUI and prompt for App ID\n  \
                  sau --id 480                      # Launch TUI with App ID 480 (skip prompt)"
)]
pub struct Args {
    /// Application ID. If provided, skips the App ID prompt in the TUI.
    /// Example: --id 480
    #[arg(short, long)]
    pub id: Option<u32>,
}
