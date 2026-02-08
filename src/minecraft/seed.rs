use rand::Rng;
use serde::Deserialize;
use serde::Serialize;

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
#[serde(untagged)]
pub enum MinecraftSeed {
    Numeric(i64),
    Text(String)
}

impl MinecraftSeed {
    pub fn random() -> MinecraftSeed {
        MinecraftSeed::Numeric(rand::rng().random::<i64>())
    }
}
