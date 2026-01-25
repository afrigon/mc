use crate::{
    cli::{commands::init::InitCommand, context::CliContext},
    utils::errors::McResult,
};

pub async fn init(context: &mut CliContext, command: &InitCommand) -> McResult<()> {
    context.shell().status("Creating", "Minecraft server")?;

    if command.path.join("mc.toml").exists() {
        anyhow::bail!("`mc init` cannot be run on existing mc server")
    }

    // if !tokio::fs::try_exists(&path).await? {
    // } else {
    //     let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
    //     utils::restricted_names::validate_server_name(name)?;

    //     tokio::fs::create_dir_all(&path).await?;
    // }

    // let path = tokio::fs::canonicalize(path).await;

    // let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
    // utils::restricted_names::validate_server_name(name)?;

    // tokio::fs::create_dir_all(&path).await?;

    // let toml_path = path.join("mc.toml");

    // if tokio::fs::try_exists(&toml_path).await? {
    //     anyhow::bail!("`mc init` cannot be run on existing mc servers")
    // }

    // // initialize default toml
    // // save toml file in directory

    // tokio::try_join!(
    //     tokio::fs::create_dir_all(path.join("minecraft")),
    //     tokio::fs::create_dir_all(path.join("java"))
    // )?;

    Ok(())
}
