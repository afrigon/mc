use std::collections::BTreeMap;
use std::io::ErrorKind;
use std::path::Path;

use anyhow::Context;
use kdl::KdlDocument;
use url::Url;
use uuid::Uuid;

use crate::manifest::raw::RawLockfile;
use crate::utils;
use crate::utils::errors::McResult;
use crate::utils::product_descriptor::RawProductDescriptor;

#[derive(Debug, Default)]
pub struct Lockfile {
    mods: Vec<ModLockfileEntry>,
    players: BTreeMap<String, Uuid>,
    changed: bool
}

impl Lockfile {
    pub fn new(mods: Vec<ModLockfileEntry>, players: BTreeMap<String, Uuid>) -> Lockfile {
        Lockfile {
            mods,
            players,
            changed: false
        }
    }

    pub fn mods(&self) -> &[ModLockfileEntry] {
        &self.mods
    }

    pub fn set_mods(&mut self, mods: Vec<ModLockfileEntry>) {
        self.mods = mods;
        self.changed = true;
    }

    pub fn players(&self) -> &BTreeMap<String, Uuid> {
        &self.players
    }

    /// Names are unique regardless of case, so a lookup matches whatever
    /// casing was recorded.
    pub fn player(&self, name: &str) -> Option<(&str, Uuid)> {
        self.players
            .iter()
            .find(|(recorded, _)| recorded.eq_ignore_ascii_case(name))
            .map(|(recorded, uuid)| (recorded.as_str(), *uuid))
    }

    pub fn record_player(&mut self, name: String, uuid: Uuid) {
        self.players.insert(name, uuid);
        self.changed = true;
    }

    /// Whether anything was recorded since the lockfile was read or built.
    pub fn changed(&self) -> bool {
        self.changed
    }

    /// A missing lockfile is an empty one.
    pub async fn read(path: &Path) -> McResult<Lockfile> {
        match tokio::fs::read_to_string(path).await {
            Ok(source) => Lockfile::from_kdl_str(&source).context("could not parse mc.lock"),
            Err(error) if error.kind() == ErrorKind::NotFound => Ok(Lockfile::default()),
            Err(error) => Err(error).context("could not read mc.lock")
        }
    }

    pub async fn write(&self, path: &Path) -> McResult<()> {
        tokio::fs::write(path, self.to_kdl_document().to_string())
            .await
            .context("could not write mc.lock")
    }

    pub fn from_kdl_str(source: &str) -> McResult<Lockfile> {
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

        let mut players = BTreeMap::new();

        for (name, entry) in raw.players {
            let uuid = entry
                .uuid
                .ok_or_else(|| anyhow::anyhow!("the locked player `{}` has no uuid", name))?
                .parse()
                .with_context(|| format!("the locked player `{}` has an invalid uuid", name))?;

            players.insert(name, uuid);
        }

        Ok(Lockfile::new(mods, players))
    }

    pub fn to_kdl_document(&self) -> KdlDocument {
        let mut document = KdlDocument::new();
        let mut modrinth = utils::kdl::node("modrinth", 0);
        let mut http = utils::kdl::node("http", 0);
        let mut players = utils::kdl::node("players", 0);

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

        for (name, uuid) in &self.players {
            let mut node = utils::kdl::node(name, 1);

            node.push(utils::kdl::quoted_property(
                "uuid",
                &uuid.hyphenated().to_string()
            ));
            players.ensure_children().nodes_mut().push(node);
        }

        for group in [modrinth, http, players] {
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
