use anyhow::Context;
use chrono::Utc;
use chrono_tz::Tz;
use serde::Serialize;

use crate::utils::errors::McResult;
use crate::utils::{self};

#[derive(Serialize)]
pub struct MinecraftEula {
    pub eula: bool
}

impl MinecraftEula {
    pub fn to_string(&self) -> McResult<String> {
        let date_string = utils::date::minecraft_date_string()?;

        Ok(format!(
            "#By changing the setting below to TRUE you are indicating your agreement to our EULA (https://aka.ms/MinecraftEULA).\n#{}\n{}",
            date_string,
            serde_java_properties::to_string(self)?
        ))
    }
}
