use std::fmt;
use std::str::FromStr;

use serde::Deserialize;

#[derive(Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TunnelProviderKind {
    Playit
}

impl fmt::Display for TunnelProviderKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let provider = match self {
            TunnelProviderKind::Playit => "playit"
        };

        write!(f, "{}", provider)
    }
}

impl FromStr for TunnelProviderKind {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "playit" | "playit.gg" => Ok(TunnelProviderKind::Playit),
            _ => anyhow::bail!("unknown tunnel provider {}", s)
        }
    }
}
