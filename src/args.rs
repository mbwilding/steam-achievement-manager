use clap::Parser;

pub fn get() -> Args {
    Args::parse()
}

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None, after_help = "Will run for all owned apps, unless id is specified")]
pub struct Args {
    /// App ID
    #[arg(short, long)]
    pub id: Option<u32>,

    /// Clear achievements
    #[arg(short, long)]
    pub clear: bool,

    /// All owned apps
    #[arg(short, long)]
    pub owned: bool,

    /// All known apps
    #[arg(short, long)]
    pub all: bool,

    /// How many apps to process at once, too high will cause issues
    #[arg(short, long, default_value = "1")]
    pub parallel: usize,

    /// App name
    #[arg(short, long, hide = true)]
    pub name: Option<String>,

    /// Worker mode
    #[arg(short, long, hide = true)]
    pub worker: bool,
}
