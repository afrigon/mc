use uuid::Uuid;

use crate::context::McContext;
use crate::manifest::lock::Lockfile;
use crate::minecraft::players;
use crate::services;
use crate::utils::errors::McResult;

pub struct ResolvedPlayer {
    pub name: String,
    pub uuid: Uuid
}

/// Turns a name into the identity the server will see. Offline-mode servers
/// derive it from the name; online-mode servers use the account's UUID,
/// remembered in the lockfile so a start never depends on the lookup.
pub async fn resolve(
    context: &McContext,
    lockfile: &mut Lockfile,
    name: &str,
    online_mode: bool
) -> McResult<ResolvedPlayer> {
    if !online_mode {
        return Ok(ResolvedPlayer {
            name: name.to_owned(),
            uuid: players::offline_uuid(name)
        });
    }

    let locked = lockfile
        .players
        .iter()
        .find(|(locked, _)| locked.eq_ignore_ascii_case(name));

    if let Some((locked_name, uuid)) = locked {
        return Ok(ResolvedPlayer {
            name: locked_name.clone(),
            uuid: *uuid
        });
    }

    let profile = services::mojang_api::get_profile(&context.http_client, name)
        .await?
        .ok_or_else(|| anyhow::anyhow!("no Minecraft account is named `{}`", name))?;

    lockfile.players.insert(profile.name.clone(), profile.id);

    Ok(ResolvedPlayer {
        name: profile.name,
        uuid: profile.id
    })
}
