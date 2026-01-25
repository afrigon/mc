use crate::{
    cli::{
        commands::minecraft::{MinecraftInstallCommand, MinecraftListCommand},
        context::CliContext,
    },
    minecraft::MinecraftVersion,
    services::minecraft_api::{
        MinecraftApiService, MinecraftApiVersionManifestEntry, MinecraftApiVersionType,
    },
    utils::errors::McResult,
};

pub async fn install(context: &mut CliContext, command: &MinecraftInstallCommand) -> McResult<()> {
    // TODO: check if already installed
    // TODO: add confirm, override, etc dialogs
    // TODO: add progress bar

    let manifest = MinecraftApiService::get_manifest(&context.http_client).await?;

    let version_id = match &command.version {
        MinecraftVersion::Latest => manifest.latest.release,
        MinecraftVersion::LatestSnapshot => manifest.latest.snapshot,
        MinecraftVersion::Version(s) => s.clone(),
    };

    if let Some(version) = manifest.versions.iter().find(|v| v.id == version_id) {
        let metadata =
            MinecraftApiService::get_metadata(&context.http_client, &version.url).await?;
        let url = &metadata.downloads.server.url;

        MinecraftApiService::download_version(&context.http_client, &url).await?;
    } else {
        // TODO: fix this
        // return Err(format!("Could not find minecraft version {}", version_id))
    };

    Ok(())
}

pub async fn list(context: &mut CliContext, command: &MinecraftListCommand) -> McResult<()> {
    let manifest = MinecraftApiService::get_manifest(&context.http_client).await?;

    let versions: Vec<&MinecraftApiVersionManifestEntry> = manifest
        .versions
        .iter()
        .filter(|v| match v.version_type {
            MinecraftApiVersionType::Release => true,
            MinecraftApiVersionType::Snapshot => command.snapshots,
            MinecraftApiVersionType::Beta => command.betas,
            MinecraftApiVersionType::Alpha => command.alphas,
        })
        .collect();

    let count = if command.all {
        versions.len()
    } else {
        10.min(versions.len())
    };

    let rest = versions.len() - count;

    for i in 0..count {
        let version = versions[i];

        print!("{}", version.id);

        if version.id == manifest.latest.release {
            print!(" (latest)")
        } else if version.id == manifest.latest.snapshot {
            print!(" (latest-snapshot)")
        }

        print!("\n");
    }

    if rest != 0 {
        println!("and {} more. use --all to see all versions", rest);
    };

    Ok(())
}
