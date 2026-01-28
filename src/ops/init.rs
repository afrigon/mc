use std::path::Path;
use std::path::PathBuf;

use crate::context::McContext;
use crate::utils;
use crate::utils::errors::McResult;

pub struct InitOptions {
    pub path: PathBuf,
    pub name: Option<String>,
    pub eula: bool
}

fn get_name<'a>(path: &'a Path, options: &'a InitOptions) -> McResult<&'a str> {
    if let Some(ref name) = options.name {
        return Ok(name);
    }

    let file_name = path.file_name().ok_or_else(|| {
        anyhow::format_err!(
            "cannot auto-detect server name from path {:?} ; use --name to override",
            path.as_os_str()
        )
    })?;

    file_name.to_str().ok_or_else(|| {
        anyhow::format_err!(
            "cannot create server with a non-unicode name: {:?}",
            file_name
        )
    })
}

pub async fn init(context: &mut McContext, options: &InitOptions) -> McResult<()> {
    let path = &options.path;
    let name = get_name(path, options)?;

    context.shell().status("Creating", "Minecraft server")?;

    let toml_path = path.join("mc.toml");

    if toml_path.exists() {
        anyhow::bail!("`mc init` cannot be run on existing mc server")
    }

    utils::restricted_names::validate_server_name(name)?;

    tokio::fs::create_dir_all(&path).await?;

    if !options.eula {
        context
            .shell()
            .warn("the server will not start until YOU agree to the Minecraft EULA (https://aka.ms/MinecraftEULA). you can do so by setting `eula = true` in `mc.toml`")?;
    }

    let mut manifest = toml_edit::DocumentMut::new();

    manifest["name"] = toml_edit::value(name);

    let server_table = manifest["server"]
        .or_insert(toml_edit::Item::Table(toml_edit::Table::new()))
        .as_table_mut()
        .ok_or_else(|| utils::errors::internal("failed to unwrap the server toml table"))?;

    server_table["eula"] = toml_edit::value(options.eula);
    server_table
        .key_mut("eula")
        .ok_or_else(|| utils::errors::internal("failed to unwrap the eula toml key"))?
        .leaf_decor_mut()
        .set_prefix("\n# Setting this to true indicates YOU have read and agree to the Minecraft EULA (https://aka.ms/MinecraftEULA).\n# This agreement is between you and Mojang/Microsoft.\n");

    manifest["backups"] = toml_edit::Item::Table(toml_edit::Table::new());
    manifest["backups"]["enabled"] = toml_edit::value(true);

    manifest["mods"] = toml_edit::Item::Table(toml_edit::Table::new());

    tokio::fs::write(toml_path, manifest.to_string()).await?;

    tokio::try_join!(
        tokio::fs::create_dir_all(path.join("minecraft")),
        tokio::fs::create_dir_all(path.join("java"))
    )?;

    context.shell().note("see more `mc.toml` keys and their definitions at https://doc.mc.frigon.app/reference/manifest.html")?;

    Ok(())
}
