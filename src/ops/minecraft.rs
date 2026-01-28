use std::path::PathBuf;

use crate::context::McContext;
use crate::minecraft::loader::LoaderKind;
use crate::services;
use crate::services::minecraft_api::MinecraftApiVersionManifestEntry;
use crate::services::minecraft_api::MinecraftApiVersionType;
use crate::utils::errors::McResult;
use crate::utils::product_descriptor::ProductDescriptor;

pub struct MinecraftInstallOptions {
    pub version: String,
    pub loader: Option<ProductDescriptor<LoaderKind>>,
    pub minecraft_directory: PathBuf
}

pub async fn install(context: &mut McContext, options: &MinecraftInstallOptions) -> McResult<()> {
    // TODO: check if already installed
    // TODO: add confirm, override, etc dialogs
    // TODO: add progress bar
    // let manifest = services::minecraft_api::get_manifest(&context.http_client).await?;

    // if let Some(version) = manifest.versions.iter().find(|v| v.id == version_id) {
    //     let metadata =
    //         services::minecraft_api::get_metadata(&context.http_client, &version.url).await?;

    //     let url = &metadata.downloads.server.url;

    //     services::minecraft_api::download_version(&context.http_client, &url).await?;
    // } else {
    //     // TODO: fix this
    //     // return Err(format!("Could not find minecraft version {}", version_id))
    // };

    Ok(())
}

pub struct MinecraftListOptions {
    pub all: bool,
    pub snapshots: bool,
    pub betas: bool,
    pub alphas: bool
}

pub async fn list(context: &mut McContext, options: &MinecraftListOptions) -> McResult<()> {
    let manifest = services::minecraft_api::get_manifest(&context.http_client).await?;

    let versions: Vec<&MinecraftApiVersionManifestEntry> = manifest
        .versions
        .iter()
        .filter(|v| match v.version_type {
            MinecraftApiVersionType::Release => true,
            MinecraftApiVersionType::Snapshot => options.snapshots,
            MinecraftApiVersionType::Beta => options.betas,
            MinecraftApiVersionType::Alpha => options.alphas
        })
        .collect();

    let count = if options.all {
        versions.len()
    } else {
        10.min(versions.len())
    };

    let rest = versions.len() - count;

    let mut shell = context.shell();
    let stdout = shell.out();

    for i in 0..count {
        let version = versions[i];

        write!(stdout, "{}", version.id);

        if version.id == manifest.latest.release {
            write!(stdout, " (latest)");
        } else if version.id == manifest.latest.snapshot {
            write!(stdout, " (latest-snapshot)");
        }

        write!(stdout, "\n");
    }

    if rest != 0 {
        writeln!(stdout, "and {} more. use --all to see all versions", rest);
    };

    Ok(())
}

pub struct MinecraftListLoadersOptions {
    pub loader: LoaderKind,
    pub minecraft_version: String,
    pub limit: usize
}

pub async fn list_loaders(
    context: &mut McContext,
    options: &MinecraftListLoadersOptions
) -> McResult<()> {
    let versions = match options.loader {
        LoaderKind::Fabric => services::fabric_api::get_versions_for_game(
            &context.http_client,
            &options.minecraft_version
        )
    }
    .await?;

    let mut shell = context.shell();
    let stdout = shell.out();

    for i in 0..options.limit {
        if i == 0 {
            writeln!(stdout, "{} (latest)", versions[i].version)?
        } else {
            writeln!(stdout, "{}", versions[i].version)?
        }
    }

    Ok(())
}
