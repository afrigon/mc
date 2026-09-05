use std::fmt;
use std::str::FromStr;

use serde::Serialize;
use serde::Serializer;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LoaderKind {
    Fabric
}

impl Default for LoaderKind {
    fn default() -> Self {
        LoaderKind::Fabric
    }
}

impl Serialize for LoaderKind {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl FromStr for LoaderKind {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "fabric" => Ok(LoaderKind::Fabric),
            _ => anyhow::bail!("loader must be fabric")
        }
    }
}

impl fmt::Display for LoaderKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            LoaderKind::Fabric => "fabric"
        };

        write!(f, "{}", s)
    }
}
