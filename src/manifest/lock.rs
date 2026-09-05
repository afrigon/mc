use std::fmt;
use std::str::FromStr;

use kdl::KdlDocument;
use url::Url;

use crate::utils;
use crate::utils::errors::McResult;
use crate::utils::product_descriptor::RawProductDescriptor;

#[derive(Debug)]
pub struct ModLockfile {
    pub mods: Vec<ModLockfileEntry>
}

impl ModLockfile {
    pub fn from_kdl_str(source: &str) -> McResult<ModLockfile> {
        let document = utils::kdl::parse_document(source)?;
        let mut mods = Vec::new();

        for node in document.nodes() {
            if node.name().value() != "mod" {
                anyhow::bail!("unknown node `{}` in the lockfile", node.name().value());
            }

            utils::kdl::check_properties(node, &["version", "source", "hash"])?;

            let name = match utils::kdl::arguments(node).as_slice() {
                [name] => name
                    .as_string()
                    .ok_or_else(|| anyhow::anyhow!("the mod name must be a string"))?,
                _ => anyhow::bail!("a `mod` node takes a single name")
            };

            let source = utils::kdl::string_property(node, "source")?
                .ok_or_else(|| anyhow::anyhow!("the mod `{}` has no source", name))?
                .parse()?;

            mods.push(ModLockfileEntry {
                name: name.to_owned(),
                version: utils::kdl::string_property(node, "version")?.map(str::to_owned),
                source,
                hash: utils::kdl::string_property(node, "hash")?.map(str::to_owned)
            });
        }

        Ok(ModLockfile { mods })
    }

    pub fn to_kdl_document(&self) -> KdlDocument {
        let mut document = KdlDocument::new();

        for entry in &self.mods {
            let mut node = utils::kdl::leaf("mod", utils::kdl::quoted(&entry.name), 0);

            if let Some(version) = &entry.version {
                node.push(utils::kdl::quoted_property("version", version));
            }

            node.push(utils::kdl::quoted_property(
                "source",
                &entry.source.to_string()
            ));

            if let Some(hash) = &entry.hash {
                node.push(utils::kdl::quoted_property("hash", hash));
            }

            document.nodes_mut().push(node);
        }

        document
    }
}

#[derive(Debug)]
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
