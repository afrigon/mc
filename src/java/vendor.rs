use serde::Deserialize;
use std::fmt;
use std::str::FromStr;

use crate::java::JavaVersion;

#[derive(Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum JavaVendor {
    correto,
    graal
}

impl JavaVendor {
    pub fn latest_version(&self) -> JavaVersion {
        JavaVersion::Java25
    }
}

impl fmt::Display for JavaVendor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let vendor = match self {
            JavaVendor::correto => "corretto",
            JavaVendor::graal => "graal"
        };

        write!(f, "{}", vendor)
    }
}

impl FromStr for JavaVendor {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "corretto" => Ok(JavaVendor::correto),
            "graal" | "graalvm" | "graal-vm" => Ok(JavaVendor::graal),
            _ => Err("unknown java vendor")
        }
    }
}
