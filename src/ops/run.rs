use std::collections::HashMap;
use std::collections::HashSet;
use std::path::PathBuf;
use std::process::Stdio;
use std::str::FromStr;

use anyhow::Context;
use tokio::process::Command;

use crate::context::McContext;
use crate::crypto::checksum::ChecksumRef;
use crate::crypto::checksum::LocalChecksum;
use crate::env::Architecture;
use crate::env::Platform;
use crate::manifest::Manifest;
use crate::manifest::lock;
use crate::manifest::lock::ModLockfile;
use crate::manifest::lock::ModLockfileSource;
use crate::manifest::raw::RawManifest;
use crate::network;
use crate::network::artifact::ArtifactKind;
use crate::network::artifact::ArtifactSource;
use crate::ops;
use crate::ops::eula::EulaOptions;
use crate::ops::init::InitDirectoriesOptions;
use crate::ops::java::JavaInstallOptions;
use crate::ops::minecraft::MinecraftInstallOptions;
use crate::ops::mods::SyncModsOptions;
use crate::services;
use crate::utils::errors::McResult;
use crate::utils::product_descriptor::RawProductDescriptor;

pub struct RunOptions {
    pub manifest_path: PathBuf,
    pub lockfile_path: PathBuf
}

fn sanitize_command(command: &Command) -> String {
    let command = command.as_std();

    let mut command_parts: Vec<String> = Vec::new();
    command_parts.push(command.get_program().to_string_lossy().into_owned());
    command_parts.extend(command.get_args().map(|a| a.to_string_lossy().into_owned()));

    command_parts
        .into_iter()
        .map(|s| {
            if s.contains(" ") || s.contains("\t") {
                format!("{:?}", s)
            } else {
                s
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

// TODO: validate error context for all cases.
// - invalid versions
// - invalid toml format
// - missing toml file
// - etc.
pub async fn run(context: &mut McContext, options: &RunOptions) -> McResult<()> {
    // TODO: make sure the server is running only once?

    let manifest_string = tokio::fs::read_to_string(&options.manifest_path)
        .await
        .context("could not find mc.toml file")?;
    let manifest_raw = toml::from_str::<RawManifest>(&manifest_string)?;

    let mut manifest = Manifest::default(context).await?;
    manifest.apply(context, &manifest_raw).await?;

    let path = context.cwd.clone();
    let instance_path = path.join("instance");

    let init_directories_options = InitDirectoriesOptions { path: path.clone() };
    ops::init::init_directories(context, &init_directories_options).await?;

    // EULA

    if manifest.server.eula == true {
        let eula_options = EulaOptions {
            accept: true,
            manifest_path: None,
            instance_path: instance_path.clone()
        };

        ops::eula::eula(context, &eula_options).await?;
    } else {
        let eula_options = EulaOptions {
            accept: false,
            manifest_path: None,
            instance_path: instance_path.clone()
        };

        ops::eula::eula(context, &eula_options).await?;

        anyhow::bail!(
            "the instance will not start until YOU agree to the Minecraft EULA (https://aka.ms/MinecraftEULA). you can do so by setting `eula = true` in `mc.toml`"
        );
    }

    // JAVA

    let java_directory = path.join("java");
    let java_path = java_directory.join(manifest.java.version.to_string());
    let current_platform = Platform::current();

    if !java_path.exists() {
        let java_install_options = JavaInstallOptions {
            architecture: Architecture::current(),
            platform: current_platform,
            version: manifest.java.version,
            java_directory
        };

        ops::java::install(context, &java_install_options).await?;
    }

    let java_bin = match current_platform {
        Platform::Windows => "javaw.exe",
        _ => "java"
    };
    let java_bin_path = java_path.join("bin").join(java_bin);

    // MINECRAFT

    let minecraft_directory = path.join("minecraft");
    let minecraft_descriptor_prefix = manifest
        .minecraft
        .loader
        .as_ref()
        .map(|l| l.to_string())
        .unwrap_or(String::from("minecraft"));

    let minecraft_descriptor = format!(
        "{}-{}",
        minecraft_descriptor_prefix, manifest.minecraft.version
    );

    let minecraft_path = minecraft_directory
        .join(minecraft_descriptor)
        .join("server.jar");

    if !minecraft_path.exists() {
        let minecraft_install_options = MinecraftInstallOptions {
            version: manifest.minecraft.version.clone(),
            loader: manifest.minecraft.loader.clone(),
            minecraft_directory
        };

        ops::minecraft::install(context, &minecraft_install_options).await?;
    }

    // TODO: fetch capabilities

    // PROPERTIES

    // MODS

    let sync_options = SyncModsOptions {
        game_version: manifest.minecraft.version.clone(),
        loader: manifest.minecraft.loader,
        lockfile_path: options.lockfile_path.clone(),
        mods_path: instance_path.join("mods")
    };

    ops::mods::sync(context, &sync_options, &manifest.mods).await?;

    // PROCESS

    let mut command = tokio::process::Command::new(java_bin_path);

    command
        .arg("-jar")
        .arg(minecraft_path.as_os_str())
        .arg("--nogui")
        .current_dir(&instance_path)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .kill_on_drop(true);

    let command_string = sanitize_command(&command);
    _ = context
        .shell()
        .status("Running", format!("`{}`", command_string));

    let mut child = command.spawn()?;

    tokio::select! {
        _ = child.wait() => {

        }
        _ = tokio::signal::ctrl_c() => {
            // TODO: rcon save + stop instead of kill
            // TODO: release the lock

            child.kill().await?
        }
    };

    // TODO: live backups

    Ok(())
}
