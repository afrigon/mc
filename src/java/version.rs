use std::fmt;
use std::str::FromStr;

use serde::Deserialize;

#[derive(Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum JavaVersion {
    Java25,
    Java21,
    Java17,
    Java11,
    Java8
}

impl JavaVersion {
    pub fn value(&self) -> u8 {
        match self {
            JavaVersion::Java25 => 25,
            JavaVersion::Java21 => 21,
            JavaVersion::Java17 => 17,
            JavaVersion::Java11 => 11,
            JavaVersion::Java8 => 8
        }
    }
}

impl fmt::Display for JavaVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let version = match self {
            JavaVersion::Java25 => "25",
            JavaVersion::Java21 => "21",
            JavaVersion::Java17 => "17",
            JavaVersion::Java11 => "11",
            JavaVersion::Java8 => "8"
        };

        write!(f, "{}", version)
    }
}

impl FromStr for JavaVersion {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "25" => Ok(JavaVersion::Java25),
            "21" => Ok(JavaVersion::Java21),
            "17" => Ok(JavaVersion::Java17),
            "11" => Ok(JavaVersion::Java11),
            "8" => Ok(JavaVersion::Java8),
            _ => anyhow::bail!("unknown java version")
        }
    }
}
