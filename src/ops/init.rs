use std::collections::HashSet;
use std::path::Path;
use std::path::PathBuf;

use crate::context::McContext;
use crate::manifest;
use crate::manifest::presets::ManifestPreset;
use crate::utils;
use crate::utils::errors::McResult;

pub struct InitDirectoriesOptions {
    pub path: PathBuf
}

pub async fn init_directories(
    _context: &mut McContext,
    options: &InitDirectoriesOptions
) -> McResult<()> {
    tokio::try_join!(
        tokio::fs::create_dir_all(options.path.join(".minecraft")),
        tokio::fs::create_dir_all(options.path.join(".java")),
        tokio::fs::create_dir_all(options.path.join("instance"))
    )?;

    Ok(())
}

const GITIGNORE: &str = "\
.DS_Store

/.java
/.minecraft
/instance
/temp

/mc.world.lock
/mc.backup.lock
";

async fn write_gitignore(context: &mut McContext, path: &Path) -> McResult<()> {
    let gitignore_path = path.join(".gitignore");

    match tokio::fs::read_to_string(&gitignore_path).await {
        Ok(existing) => {
            let existing_entries: HashSet<&str> = existing.lines().map(str::trim).collect();
            let missing: Vec<&str> = GITIGNORE
                .lines()
                .filter(|entry| !entry.is_empty() && !existing_entries.contains(entry))
                .collect();

            if !missing.is_empty() {
                context.shell().warn(format!(
                    ".gitignore already exists and was left untouched, but is missing entries: {}",
                    missing.join(", ")
                ))?;
            }

            Ok(())
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            tokio::fs::write(&gitignore_path, GITIGNORE).await?;
            Ok(())
        }
        Err(error) => Err(error.into())
    }
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

    write_gitignore(context, path).await?;

    let init_directories_options = InitDirectoriesOptions {
        path: options.path.clone()
    };
    init_directories(context, &init_directories_options).await?;

    context.shell().note("see more `mc.toml` keys and their definitions at https://doc.mc.frigon.app/reference/manifest.html")?;

    Ok(())
}
