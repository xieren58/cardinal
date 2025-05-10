use clap::Parser;

#[derive(Parser)]
pub struct Cli {
    #[clap(short, long, default_value = "false")]
    /// Open enabled, cache was ignored and filesystem will be rewalked.
    pub refresh: bool,
}
