#[derive(Debug, Clone, Copy)]
pub struct Verbosity(pub u8);

impl Verbosity {
    pub fn directive(&self) -> &str {
        match self.0 - 1 {
            0 => "error",
            1 => "warn",
            2 => "info",
            3 => "debug",
            _ => "trace",
        }
    }
}

impl std::str::FromStr for Verbosity {
    type Err = std::num::ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse::<u8>().map(Verbosity)
    }
}

impl std::fmt::Display for Verbosity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
