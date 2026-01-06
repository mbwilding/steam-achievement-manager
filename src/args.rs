use clap::Parser;

pub fn get() -> Args {
    Args::parse()
}

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None, after_help = "You can combine arguments, for example --id 123 --clear")]
pub struct Args {
    /// Application ID(s). You can specify multiple IDs by using the flag multiple times or by separating IDs with commas in a single flag.
    #[arg(short, long, value_delimiter = ',')]
    pub id: Vec<u32>,

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
