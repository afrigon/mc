use std::str::FromStr;

use serde::Serialize;
use serde::Serializer;

pub mod eula;
pub mod log4j;
pub mod seed;
pub mod server_properties;

#[derive(Serialize, Copy, Clone)]
#[serde(rename_all = "lowercase")]
pub enum MinecraftDifficulty {
    Peaceful,
    Easy,
    Normal,
    Hard
}

impl Default for MinecraftDifficulty {
    fn default() -> Self {
        MinecraftDifficulty::Normal
    }
}

impl FromStr for MinecraftDifficulty {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "peaceful" => Ok(MinecraftDifficulty::Peaceful),
            "easy" => Ok(MinecraftDifficulty::Easy),
            "normal" => Ok(MinecraftDifficulty::Normal),
            "hard" => Ok(MinecraftDifficulty::Hard),
            _ => anyhow::bail!("difficulty must be peaceful, easy, normal, or hard")
        }
    }
}

#[derive(Serialize, Copy, Clone)]
#[serde(rename_all = "lowercase")]
pub enum MinecraftGamemode {
    Survival,
    Creative,
    Adventure,
    Spectator
}

impl Default for MinecraftGamemode {
    fn default() -> Self {
        MinecraftGamemode::Survival
    }
}

impl FromStr for MinecraftGamemode {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "survival" => Ok(MinecraftGamemode::Survival),
            "creative" => Ok(MinecraftGamemode::Creative),
            "adventure" => Ok(MinecraftGamemode::Adventure),
            "spectator" => Ok(MinecraftGamemode::Spectator),
            _ => anyhow::bail!("gamemode must be survival, creative, adventure, or spectator")
        }
    }
}

#[derive(Serialize, Copy, Clone)]
#[allow(non_camel_case_types, dead_code)]
pub enum MinecraftRegionCompression {
    deflate,
    lz4
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[allow(dead_code)]
#[repr(u8)]
pub enum MinecraftPermission {
    /// No permission.
    All = 0,

    /// The player can bypass spawn protection.
    Moderator = 1,

    /// - More commands are available.
    /// - The player can use command blocks.
    /// - The player can copy the server-side NBT data of an entity or a block entity when pressing the F3 + I debug hotkey, and copy the client-side NBT data when pressing ⇧ Shift + F3 + I.
    /// - The player can use F3 + F4 (game mode switcher) and F3 + N debug hotkey (toggle between Spectator and the previous game mode).
    /// - The player can change or lock difficulty in Options screen. Note that the player in a singleplayer world or the owner of a LAN world can change or lock difficulty without a permission level of 2.
    /// - With "Operator Items Tab" option turned on, the player can find operator items and an "Operator Utilities" tab in the creative inventory.
    /// - Target selectors can be used in commands like /tell and raw JSON texts.
    Gamemaster = 2,

    /// Commands related to multiplayer management are available.
    Admin = 3,

    /// All commands are available, including commands related to server management.
    Owner = 4
}

impl Serialize for MinecraftPermission {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer
    {
        serializer.serialize_u8(*self as u8)
    }
}

#[derive(Serialize, Copy, Clone)]
pub enum MinecraftLevelKind {
    #[serde(rename = "minecraft:normal")]
    Normal,

    #[serde(rename = "minecraft:flat")]
    Flat,

    #[serde(rename = "minecraft:large_biomes")]
    LargeBiomes,

    #[serde(rename = "minecraft:amplified")]
    Amplified,

    #[serde(rename = "minecraft:single_biome_surface")]
    SingleBiomeSurface
}

impl Default for MinecraftLevelKind {
    fn default() -> Self {
        MinecraftLevelKind::Normal
    }
}

impl FromStr for MinecraftLevelKind {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "minecraft:normal" => Ok(MinecraftLevelKind::Normal),
            "minecraft:flat" => Ok(MinecraftLevelKind::Flat),
            "minecraft:large_biomes" => Ok(MinecraftLevelKind::LargeBiomes),
            "minecraft:amplified" => Ok(MinecraftLevelKind::Amplified),
            "minecraft:single_biome_surface" => Ok(MinecraftLevelKind::SingleBiomeSurface),
            _ => anyhow::bail!(
                "level type must be minecraft:normal, minecraft:flat, minecraft:large_biomes, minecraft:amplified, or minecraft:single_biome_surface"
            )
        }
    }
}
