use kdl::KdlDocument;
use url::Url;

use crate::manifest::raw::RawLockfile;
use crate::utils;
use crate::utils::errors::McResult;
use crate::utils::product_descriptor::RawProductDescriptor;

#[derive(Debug)]
pub struct ModLockfile {
    pub mods: Vec<ModLockfileEntry>
}

impl ModLockfile {
    pub fn from_kdl_str(source: &str) -> McResult<ModLockfile> {
        let raw: RawLockfile = utils::kdl::deserialize(source)?;
        let mut mods = Vec::new();

        for (name, entry) in raw.modrinth {
            mods.push(ModLockfileEntry {
                name,
                version: entry.version,
                source: ModLockfileSource::Modrinth,
                hash: entry.hash
            });
        }

        for (name, entry) in raw.http {
            let url = entry
                .url
                .ok_or_else(|| anyhow::anyhow!("the locked mod `{}` has no url", name))?;

            mods.push(ModLockfileEntry {
                name,
                version: entry.version,
                source: ModLockfileSource::Url(url),
                hash: entry.hash
            });
        }

        Ok(ModLockfile { mods })
    }

    pub fn to_kdl_document(&self) -> KdlDocument {
        let mut document = KdlDocument::new();
        let mut modrinth = utils::kdl::node("modrinth", 0);
        let mut http = utils::kdl::node("http", 0);

        for entry in &self.mods {
            let mut node = utils::kdl::node(&entry.name, 1);

            match &entry.source {
                ModLockfileSource::Modrinth => {
                    if let Some(version) = &entry.version {
                        node.push(utils::kdl::quoted_property("version", version));
                    }

                    if let Some(hash) = &entry.hash {
                        node.push(utils::kdl::quoted_property("hash", hash));
                    }

                    modrinth.ensure_children().nodes_mut().push(node);
                }
                ModLockfileSource::Url(url) => {
                    node.push(utils::kdl::quoted_property("url", url.as_str()));

                    if let Some(hash) = &entry.hash {
                        node.push(utils::kdl::quoted_property("hash", hash));
                    }

                    http.ensure_children().nodes_mut().push(node);
                }
            }
        }

        for group in [modrinth, http] {
            if group.children().is_some() {
                let mut group = group;

                if !document.nodes().is_empty() {
                    utils::kdl::add_blank_line_before(&mut group);
                }

                document.nodes_mut().push(group);
            }
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
