use clap::Parser;

pub fn get() -> Args {
    Args::parse()
}

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None, after_help = "You can combine arguments, for example --id 123 --clear")]
pub struct Args {
    /// App ID
    #[arg(short, long)]
    pub id: Option<u32>,

    /// Clear achievements
    #[arg(short, long)]
    pub clear: bool,

    /// How many apps to process at once, too high will cause issues
    #[arg(short, long, default_value = "1")]
    pub parallel: usize,

    /// Worker mode
    #[arg(short, long, hide = true)]
    pub worker: bool,
}
