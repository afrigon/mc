use std::path::PathBuf;

use crate::context::McContext;
use crate::minecraft::loader::LoaderKind;
use crate::network;
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
    let prefix = options
        .loader
        .as_ref()
        .map(|l| l.to_string())
        .unwrap_or(String::from("minecraft"));

    let name = format!("{}-{}", prefix, options.version);
    let directory = options.minecraft_directory.join(&name);

    let path = directory.join("server.jar");

    if path.exists() {
        anyhow::bail!("{} is already installed", name);
    }

    // TODO: add progress bar

    _ = context.shell().status("Installing", name);

    tokio::fs::create_dir_all(&directory).await?;

    // TODO: use different api based on loader
    let source = if let Some(ref loader) = options.loader {
        match loader.product {
            LoaderKind::Fabric => {
                services::fabric_api::artifact_source(
                    &context.http_client,
                    loader,
                    &options.version
                )
                .await?
            }
        }
    } else {
        services::minecraft_api::artifact_source(&context.http_client, &options.version).await?
    };

    network::stream_artifact(&context.http_client, source, &path).await
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

        _ = write!(stdout, "{}", version.id);

        if version.id == manifest.latest.release {
            _ = write!(stdout, " (latest)");
        } else if version.id == manifest.latest.snapshot {
            _ = write!(stdout, " (latest-snapshot)");
        }

        _ = write!(stdout, "\n");
    }

    if rest != 0 {
        _ = writeln!(stdout, "and {} more. use --all to see all versions", rest);
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
