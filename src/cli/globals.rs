use clap::ArgAction;
use clap::Args;
use clap::ColorChoice;

#[derive(Args)]
pub struct GlobalOptions {
    /// Use verbose output (-vv very verbose output)
    #[arg(short, long, global = true, action = ArgAction::Count)]
    pub verbose: u8,

    /// Do not print mc log messages
    #[arg(short, long, global = true)]
    pub quiet: bool,

    /// Coloring: auto, always, never
    #[arg(
        long, 
        value_name = "WHEN", 
        global = true, 
        default_value_t = ColorChoice::Auto, 
        hide_default_value = true,
        hide_possible_values = true,
        ignore_case = true
    )]
    pub color: ColorChoice
}
