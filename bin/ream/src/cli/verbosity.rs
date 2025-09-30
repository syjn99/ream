// Fixed verbosity levels for specific crates to reduce log noise
const ACTIX_SERVER_DIRECTIVE: &str = "actix_server=warn";
const DISCV5_DIRECTIVE: &str = "discv5=error";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verbosity {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl Verbosity {
    pub fn directive(&self) -> String {
        let directive = match self {
            Verbosity::Error => "error",
            Verbosity::Warn => "warn",
            Verbosity::Info => "info",
            Verbosity::Debug => "debug",
            Verbosity::Trace => "trace",
        };
        format!("{directive},{ACTIX_SERVER_DIRECTIVE},{DISCV5_DIRECTIVE}")
    }
}

pub fn verbosity_parser(s: &str) -> Result<Verbosity, String> {
    let level = s.parse::<u8>().map_err(|err| err.to_string())?;

    if !(1..=5).contains(&level) {
        return Err(format!("verbosity must be between 1 and 5, got {level}"));
    }

    Ok(match level {
        1 => Verbosity::Error,
        2 => Verbosity::Warn,
        3 => Verbosity::Info,
        4 => Verbosity::Debug,
        5 => Verbosity::Trace,
        _ => unreachable!(),
    })
}
