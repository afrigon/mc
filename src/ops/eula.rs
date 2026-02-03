use std::path::PathBuf;

use anyhow::Context;
use chrono::Utc;
use chrono_tz::Tz;

use crate::context::McContext;
use crate::utils;
use crate::utils::errors::McResult;

pub struct EulaOptions {
    pub accept: bool,
    pub manifest_path: Option<PathBuf>,
    pub instance_path: PathBuf
}

pub async fn eula(context: &mut McContext, options: &EulaOptions) -> McResult<()> {
    if let Some(manifest_path) = options.manifest_path.clone() {
        // TODO: edit the manifest
    }

    let eula_path = options.instance_path.join("eula.txt");
    let mut eula = toml_edit::DocumentMut::new();

    let mut eula_value = toml_edit::value(options.accept);
    eula_value
        .as_value_mut()
        .unwrap()
        .decor_mut()
        .set_prefix("");
    eula["eula"] = eula_value;

    // TODO: should I recycle the date from an existing eula.txt?
    let tz_string = iana_time_zone::get_timezone().context("could not get current timezone")?;
    let tz: Tz = tz_string.parse().context("could not parse iana timezone")?;
    let date = Utc::now().with_timezone(&tz);
    let date_string = date.format("%a %b %e %H:%M:%S %Z %Y").to_string();

    let mut key = eula
        .key_mut("eula")
        .ok_or_else(|| utils::errors::internal("failed to unwrap the eula toml key"))?;

    let decor = key.leaf_decor_mut();
    decor.set_prefix(format!("#By changing the setting below to TRUE you are indicating your agreement to our EULA (https://aka.ms/MinecraftEULA).\n#{}\n", date_string));
    decor.set_suffix("");

    tokio::fs::write(eula_path, eula.to_string()).await?;

    Ok(())
}
