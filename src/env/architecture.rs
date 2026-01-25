use std::convert::Infallible;
use std::env::consts::ARCH;
use std::fmt;
use std::str::FromStr;

#[allow(non_camel_case_types)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Architecture {
    x86_64,
    aarch64,
    Unknown
}

impl Architecture {
    pub fn current() -> Architecture {
        match ARCH {
            "aarch64" => Architecture::aarch64,
            "x86_64" => Architecture::x86_64,
            _ => Architecture::Unknown
        }
    }
}

impl FromStr for Architecture {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Architecture, Infallible> {
        match s {
            "x86_64" => Ok(Architecture::x86_64),
            "aarch64" => Ok(Architecture::aarch64),
            _ => Ok(Architecture::Unknown)
        }
    }
}

impl fmt::Display for Architecture {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Architecture::x86_64 => "x86_64",
            Architecture::aarch64 => "aarch64",
            Architecture::Unknown => "unknown"
        };

        write!(f, "{}", value)
    }
}
