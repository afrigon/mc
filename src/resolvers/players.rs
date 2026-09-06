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

/// Turns a name into the identity the server will see. An offline-mode
/// server derives it from the name alone, so the lockfile, which remembers
/// account identities, only applies to an online-mode server.
pub async fn resolve(
    context: &McContext,
    lockfile: &mut Lockfile,
    name: &str,
    online_mode: bool
) -> McResult<ResolvedPlayer> {
    if online_mode {
        resolve_account(context, lockfile, name).await
    } else {
        Ok(ResolvedPlayer {
            name: name.to_owned(),
            uuid: players::offline_uuid(name)
        })
    }
}

async fn resolve_account(
    context: &McContext,
    lockfile: &mut Lockfile,
    name: &str
) -> McResult<ResolvedPlayer> {
    if let Some((recorded, uuid)) = lockfile.player(name) {
        return Ok(ResolvedPlayer {
            name: recorded.to_owned(),
            uuid
        });
    }

    let profile = services::mojang_api::get_profile(&context.http_client, name)
        .await?
        .ok_or_else(|| anyhow::anyhow!("no Minecraft account is named `{}`", name))?;

    lockfile.record_player(profile.name.clone(), profile.id);

    Ok(ResolvedPlayer {
        name: profile.name,
        uuid: profile.id
    })
}
