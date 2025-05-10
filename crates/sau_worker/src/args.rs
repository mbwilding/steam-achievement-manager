use clap::Parser;

pub fn get() -> Args {
    Args::parse()
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// App ID
    #[arg(short, long)]
    pub id: u32,

    /// App name
    #[arg(short, long, default_value = "Unspecified")]
    pub name: String,

    /// Clear achievements
    #[arg(short, long)]
    pub clear: bool,
}
