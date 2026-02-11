use serde::Deserialize;
use serde::Serialize;
use serde::Serializer;

pub mod eula;
pub mod seed;
pub mod server_properties;

#[derive(Serialize, Deserialize, Copy, Clone)]
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

#[derive(Serialize, Deserialize, Copy, Clone)]
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

#[derive(Serialize, Copy, Clone)]
#[allow(non_camel_case_types)]
pub enum MinecraftRegionCompression {
    deflate,
    lz4
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
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

#[derive(Deserialize, Serialize, Copy, Clone)]
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
