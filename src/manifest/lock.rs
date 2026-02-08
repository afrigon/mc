use std::fmt;
use std::str::FromStr;

use anyhow::Context;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;
use url::Url;

use crate::utils::product_descriptor::RawProductDescriptor;

#[derive(Debug, Serialize, Deserialize)]
pub struct ModLockfile {
    pub mods: Vec<ModLockfileEntry>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModLockfileEntry {
    pub name: String,
    pub version: Option<String>,
    pub source: ModLockfileSource,
    pub hash: Option<String>
}

impl ModLockfileEntry {
    pub fn descriptor(&self) -> RawProductDescriptor {
        RawProductDescriptor {
            product: self.name.clone(),
            version: self.version.clone()
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModLockfileSource {
    Modrinth,
    Url(Url)
}

impl Serialize for ModLockfileSource {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for ModLockfileSource {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>
    {
        let s = String::deserialize(deserializer)?;

        ModLockfileSource::from_str(&s)
            .with_context(|| format!("could not parse lockfile source: {s}"))
            .map_err(serde::de::Error::custom)
    }
}

impl FromStr for ModLockfileSource {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "modrinth" {
            return Ok(ModLockfileSource::Modrinth);
        }

        let (prefix, data) = s
            .split_once('+')
            .ok_or_else(|| anyhow::anyhow!("could not parse lockfile source"))?;

        match prefix {
            "url" => {
                let url = Url::parse(data)?;
                Ok(ModLockfileSource::Url(url))
            }
            _ => anyhow::bail!("unsupported prefix {} in lockfile source", prefix)
        }
    }
}

impl fmt::Display for ModLockfileSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            ModLockfileSource::Modrinth => "modrinth".to_string(),
            ModLockfileSource::Url(url) => format!("url+{}", url)
        };

        write!(f, "{}", s)
    }
}
