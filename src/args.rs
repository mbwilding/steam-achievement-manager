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
                  sau --id 480 --clear              # Clear all achievements for app 480\n  \
                  sau --id 480,570 --tui            # Launch TUI for multiple apps\n  \
                  sau --id 480 --tui                # Interactive selection for app 480"
)]
pub struct Args {
    /// Application ID(s). You can specify multiple IDs by using the flag multiple times or by separating IDs with commas in a single flag.
    /// Example: --id 480 or --id 480,570,220
    #[arg(short, long, value_delimiter = ',')]
    pub id: Vec<u32>,

    /// Clear achievements instead of setting them. In CLI mode, clears all achievements. In TUI mode, this flag is ignored (use 'm' key to toggle mode)
    #[arg(short, long)]
    pub clear: bool,

    /// Launch TUI (Text User Interface) mode for interactive achievement selection.
    /// Allows you to select specific achievements to set/clear with checkboxes.
    /// Only supports a single app ID at a time.
    #[arg(short, long)]
    pub tui: bool,

    /// How many apps to process at once, too high will cause issues
    #[arg(short, long, default_value = "1")]
    pub parallel: usize,

    /// Worker mode
    #[arg(short, long, hide = true)]
    pub worker: bool,
}
