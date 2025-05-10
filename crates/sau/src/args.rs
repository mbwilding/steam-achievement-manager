use clap::Parser;

pub fn get_and_validate() -> Args {
    let args = Args::parse();

    if !args.all && args.id.is_none() {
        println!("Specify either the id or all flag, or --help");
        std::process::exit(1);
    }

    args
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// App ID
    #[arg(short, long)]
    pub id: Option<u32>,

    /// App name
    #[arg(short, long, hide = true)]
    pub name: Option<String>,

    /// Run for all apps
    #[arg(short, long)]
    pub all: bool,

    /// Clear achievements
    #[arg(short, long)]
    pub clear: bool,

    /// How many games at once, too high will cause issues
    #[arg(short, long, default_value = "1")]
    pub parallel: usize,
}
