use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::VecDeque;
use std::path::PathBuf;
use std::str::FromStr;

use anyhow::Context;

use crate::context::McContext;
use crate::crypto::checksum::ChecksumRef;
use crate::crypto::checksum::LocalChecksum;
use crate::manifest::Manifest;
use crate::manifest::ManifestMod;
use crate::manifest::lock::ModLockfile;
use crate::manifest::lock::ModLockfileEntry;
use crate::manifest::lock::ModLockfileSource;
use crate::manifest::raw::RawManifest;
use crate::mods::loader::LoaderKind;
use crate::network;
use crate::network::artifact::ArtifactKind;
use crate::network::artifact::ArtifactSource;
use crate::services;
use crate::utils::errors::McResult;
use crate::utils::product_descriptor::ProductDescriptor;
use crate::utils::product_descriptor::RawProductDescriptor;

pub struct AddModsOptions {
    pub manifest_path: PathBuf,
    pub lockfile_path: PathBuf
}

pub async fn add(context: &mut McContext, options: &AddModsOptions) -> McResult<()> {
    let manifest_string = tokio::fs::read_to_string(&options.manifest_path)
        .await
        .context("could not find mc.toml file")?;
    let manifest_raw = toml::from_str::<RawManifest>(&manifest_string)?;

    let mut manifest = Manifest::default(context).await?;
    manifest.apply(context, &manifest_raw).await?;

    let path = context.cwd.clone();
    let instance_path = path.join("instance");

    let sync_options = SyncModsOptions {
        game_version: manifest.minecraft.version.clone(),
        loader: manifest.minecraft.loader,
        lockfile_path: options.lockfile_path.clone(),
        mods_path: instance_path.join("mods")
    };

    sync(context, &sync_options, &manifest.mods).await?;

    Ok(())
}

pub struct RemoveModsOptions {
    pub name: String,
    pub manifest_path: PathBuf,
    pub lockfile_path: PathBuf
}

pub async fn remove(context: &mut McContext, options: &RemoveModsOptions) -> McResult<()> {
    let manifest_string = tokio::fs::read_to_string(&options.manifest_path)
        .await
        .context("could not find mc.toml file")?;
    let manifest_raw = toml::from_str::<RawManifest>(&manifest_string)?;

    let mut manifest = Manifest::default(context).await?;
    manifest.apply(context, &manifest_raw).await?;

    let path = context.cwd.clone();
    let instance_path = path.join("instance");

    let sync_options = SyncModsOptions {
        game_version: manifest.minecraft.version.clone(),
        loader: manifest.minecraft.loader,
        lockfile_path: options.lockfile_path.clone(),
        mods_path: instance_path.join("mods")
    };

    sync(context, &sync_options, &manifest.mods).await?;

    Ok(())
}

pub struct UpdateModsOptions {
    pub manifest_path: PathBuf,
    pub lockfile_path: PathBuf
}

pub async fn update(context: &mut McContext, options: &UpdateModsOptions) -> McResult<()> {
    let manifest_string = tokio::fs::read_to_string(&options.manifest_path)
        .await
        .context("could not find mc.toml file")?;
    let manifest_raw = toml::from_str::<RawManifest>(&manifest_string)?;

    let mut manifest = Manifest::default(context).await?;
    manifest.apply(context, &manifest_raw).await?;

    let path = context.cwd.clone();
    let instance_path = path.join("instance");

    let sync_options = SyncModsOptions {
        game_version: manifest.minecraft.version.clone(),
        loader: manifest.minecraft.loader,
        lockfile_path: options.lockfile_path.clone(),
        mods_path: instance_path.join("mods")
    };

    sync(context, &sync_options, &manifest.mods).await?;

    Ok(())
}

pub struct SyncModsOptions {
    pub game_version: String,
    pub loader: Option<ProductDescriptor<LoaderKind>>,
    pub mods_path: PathBuf,
    pub lockfile_path: PathBuf
}

pub async fn sync(
    context: &mut McContext,
    options: &SyncModsOptions,
    mods: &HashMap<String, ManifestMod>
) -> McResult<()> {
    if let Some(ref loader) = options.loader {
        tokio::fs::create_dir_all(&options.mods_path).await?;

        let mut new_lockfile =
            flatten(context, mods, &loader.product, &options.game_version).await?;

        let old_lockfile = tokio::fs::read_to_string(&options.lockfile_path)
            .await
            .ok()
            .and_then(|s| toml::from_str::<ModLockfile>(&s).ok())
            .map(|lockfile| lockfile.mods)
            .unwrap_or_default();

        for new in &mut new_lockfile {
            for old in &old_lockfile {
                if old.name == new.name && old.version == new.version {
                    new.hash = old.hash.clone();

                    break;
                }
            }
        }

        // TODO: double check filename includes hash when using url source

        let mut extra_mods = HashSet::new();
        let mut rd = tokio::fs::read_dir(&options.mods_path).await?;

        while let Some(ref entry) = rd.next_entry().await? {
            let path = entry.path();
            let stem = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or_default();

            let product = RawProductDescriptor::from_str(stem)?;

            extra_mods.insert(product);
        }

        for new in &new_lockfile {
            let descriptor = &new.descriptor();

            // if already installed
            if extra_mods.contains(descriptor) {
                // TODO: should the lockfile hash be validated here?
                extra_mods.remove(descriptor);
            } else {
                let name = format!("{}", descriptor);
                _ = context.shell().status("Adding", &name);

                let output = options.mods_path.join(&name).with_extension("jar");

                match &new.source {
                    ModLockfileSource::Modrinth => {
                        let version_string = new.version.clone().ok_or_else(|| {
                            anyhow::anyhow!(
                                "could not installed modrinth mod without a specific version"
                            )
                        })?;

                        let version = services::modrinth_api::get_version(
                            &context.http_client,
                            &version_string
                        )
                        .await?;

                        let file = version.files.iter().find(|f| f.primary).ok_or_else(|| {
                            anyhow::anyhow!("could not find a file to install for {}", name)
                        })?;

                        let mut checksum = [0u8; 20];

                        _ = hex::decode_to_slice(&file.hashes.sha1, &mut checksum);

                        let source = ArtifactSource {
                            url: file.url.clone(),
                            kind: ArtifactKind::Jar,
                            checksum: Some(ChecksumRef::Local(LocalChecksum::sha1(checksum)))
                        };

                        network::stream_artifact(&context.http_client, source, &output).await?;

                        // TODO: hash into lockfile
                    }
                    ModLockfileSource::Url(url) => {
                        let source = ArtifactSource {
                            url: url.clone(),
                            kind: ArtifactKind::Jar,
                            checksum: None
                        };

                        network::stream_artifact(&context.http_client, source, &output).await?;

                        // TODO: hash into lockfile
                    }
                }
            }
        }

        for descriptor in extra_mods {
            let name = format!("{}", descriptor);
            _ = context.shell().status("Removing", &name);

            tokio::fs::remove_file(options.mods_path.join(name).with_extension("jar")).await?;
        }

        let lockfile = ModLockfile { mods: new_lockfile };
        let lockfile_string = toml::to_string_pretty(&lockfile)?;
        tokio::fs::write(&options.lockfile_path, lockfile_string).await?;
    } else {
        if !mods.is_empty() {
            _ = context
                .shell()
                .warn("a loader must be set to enable mods, ignoring all mods.");
        }
    }

    Ok(())
}

pub async fn flatten(
    context: &mut McContext,
    mods: &HashMap<String, ManifestMod>,
    loader: &LoaderKind,
    game_version: &String
) -> McResult<Vec<ModLockfileEntry>> {
    let mut resolved_mods = Vec::new();
    let mut queue = VecDeque::new();
    let mut seen = HashSet::new();

    for (name, m) in mods {
        match m {
            ManifestMod::Detailed(m) => {
                queue.push_back((name.clone(), Some(m.version.clone())));
            }
            ManifestMod::Remote(url) => {
                resolved_mods.push(ModLockfileEntry {
                    name: name.clone(),
                    version: None,
                    source: ModLockfileSource::Url(url.clone()),
                    hash: None
                });
            }
        }
    }

    while let Some((name, version)) = queue.pop_front() {
        if seen.contains(&name) {
            continue;
        }

        seen.insert(name.clone());

        let v = if let Some(ref version) = version {
            services::modrinth_api::get_version(&context.http_client, version).await?
        } else {
            services::modrinth_api::get_latest_version(
                &context.http_client,
                &name,
                loader,
                game_version
            )
            .await?
        };

        resolved_mods.push(ModLockfileEntry {
            name,
            version: Some(v.id.clone()),
            source: ModLockfileSource::Modrinth,
            hash: None
        });

        for dependency in v.dependencies {
            let project =
                services::modrinth_api::get_project(&context.http_client, &dependency.project_id)
                    .await?;

            queue.push_back((project.slug, dependency.version_id));
        }
    }

    resolved_mods.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(resolved_mods)
}
