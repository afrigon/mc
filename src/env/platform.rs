use std::convert::Infallible;
use std::env::consts::OS;
use std::fmt;
use std::str::FromStr;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    Windows,
    Linux,
    MacOS,
    Unknown
}

impl Platform {
    pub fn current() -> Platform {
        match OS {
            "linux" => Platform::Linux,
            "windows" => Platform::Windows,
            "macos" => Platform::MacOS,
            _ => Platform::Unknown
        }
    }
}

impl FromStr for Platform {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Platform, Infallible> {
        match s {
            "windows" => Ok(Platform::Windows),
            "linux" => Ok(Platform::Linux),
            "macos" => Ok(Platform::MacOS),
            _ => Ok(Platform::Unknown)
        }
    }
}

impl fmt::Display for Platform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Platform::Windows => "windows",
            Platform::Linux => "linux",
            Platform::MacOS => "macos",
            Platform::Unknown => "unknown"
        };

        write!(f, "{}", value)
    }
}
