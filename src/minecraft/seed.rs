use serde::Serialize;

#[derive(Serialize, Clone, PartialEq, Eq, Debug)]
#[serde(untagged)]
pub enum MinecraftSeed {
    Numeric(i64),
    Text(String)
}
