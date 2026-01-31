use rand::Rng;
use serde::Deserialize;

#[derive(Deserialize, Clone, PartialEq, Eq, Debug)]
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
