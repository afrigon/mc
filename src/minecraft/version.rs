use std::str::FromStr;

#[derive(Clone)]
pub enum MinecraftVersion {
    Latest,
    LatestSnapshot,
    Version(String)
}

impl FromStr for MinecraftVersion {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "latest" => Ok(MinecraftVersion::Latest),
            "latest-snapshot" => Ok(MinecraftVersion::LatestSnapshot),
            v => Ok(MinecraftVersion::Version(v.to_owned()))
        }
    }
}
