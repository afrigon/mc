use std::convert::Infallible;
use std::path::Path;
use std::path::PathBuf;
use std::str::FromStr;

use crate::context::McContext;
use crate::manifest;
use crate::manifest::presets::ManifestPreset;
use crate::utils;
use crate::utils::errors::McResult;

pub struct InitDirectoriesOptions {
    pub path: PathBuf
}

pub async fn init_directories(
    context: &mut McContext,
    options: &InitDirectoriesOptions
) -> McResult<()> {
    tokio::try_join!(
        tokio::fs::create_dir_all(options.path.join("minecraft")),
        tokio::fs::create_dir_all(options.path.join("java")),
        tokio::fs::create_dir_all(options.path.join("instance"))
    )?;

    Ok(())
}

pub struct InitOptions {
    pub path: PathBuf,
    pub name: Option<String>,
    pub eula: bool,
    pub preset: ManifestPreset
}

fn get_name<'a>(path: &'a Path, options: &'a InitOptions) -> McResult<&'a str> {
    if let Some(ref name) = options.name {
        return Ok(name);
    }

    let file_name = path.file_name().ok_or_else(|| {
        anyhow::format_err!(
            "cannot auto-detect instance name from path {:?} ; use --name to override",
            path.as_os_str()
        )
    })?;

    file_name.to_str().ok_or_else(|| {
        anyhow::format_err!(
            "cannot create instance with a non-unicode name: {:?}",
            file_name
        )
    })
}

pub async fn init(context: &mut McContext, options: &InitOptions) -> McResult<()> {
    let path = &options.path;
    let name = get_name(path, options)?;

    context.shell().status("Creating", "Minecraft instance")?;

    let toml_path = path.join("mc.toml");

    if toml_path.exists() {
        anyhow::bail!("`mc init` cannot be run on existing mc instance")
    }

    utils::restricted_names::validate_instance_name(name)?;

    tokio::fs::create_dir_all(&path).await?;

    if !options.eula {
        context
            .shell()
            .warn("the instance will not start until YOU agree to the Minecraft EULA (https://aka.ms/MinecraftEULA). you can do so by setting `eula = true` in the server section of `mc.toml`")?;
    }

    let manifest =
        manifest::presets::create_document(context, options.preset, name, options.eula).await?;

    tokio::fs::write(toml_path, manifest.to_string()).await?;

    let init_directories_options = InitDirectoriesOptions {
        path: options.path.clone()
    };
    init_directories(context, &init_directories_options).await?;

    context.shell().note("see more `mc.toml` keys and their definitions at https://doc.mc.frigon.app/reference/manifest.html")?;

    Ok(())
}
