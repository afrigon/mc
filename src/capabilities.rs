use std::collections::HashSet;

use crate::services::minecraft_api::MinecraftApiVersionManifestEntry;
use crate::services::minecraft_api::MinecraftApiVersionType;

#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
pub enum Capability {
    /// Remote console (RCON) is a TCP/IP-based protocol that allows server administrators to remotely execute commands.
    /// Note: Introduced in Java Edition Beta 1.9 Prerelease 4
    RemoteConsole,
    ServerManagementProtocol
}

pub fn from_minecraft_version(version: MinecraftApiVersionManifestEntry) -> HashSet<Capability> {
    let mut capabilities = HashSet::new();

    // TODO: figure out a way to order minecraft versions, the naming scheme is all over the place

    if version.version_type != MinecraftApiVersionType::Alpha
        && version.version_type != MinecraftApiVersionType::Beta
    {
        capabilities.insert(Capability::RemoteConsole);
    }

    capabilities
}
