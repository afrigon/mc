use std::fmt;
use std::str::FromStr;

use serde::Deserialize;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Deserialize)]
pub enum ModServiceKind {
    Modrinth
}

impl Default for ModServiceKind {
    fn default() -> Self {
        ModServiceKind::Modrinth
    }
}

impl FromStr for ModServiceKind {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "modrinth" => Ok(ModServiceKind::Modrinth),
            _ => anyhow::bail!("mod service must be modrinth")
        }
    }
}

impl fmt::Display for ModServiceKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            ModServiceKind::Modrinth => "modrinth"
        };

        write!(f, "{}", s)
    }
}
