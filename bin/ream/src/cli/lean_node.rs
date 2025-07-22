use clap::Parser;

#[derive(Debug, Parser)]
pub struct LeanNodeConfig {
    /// Verbosity level
    #[arg(short, long, default_value_t = 3)]
    pub verbosity: u8,
}
