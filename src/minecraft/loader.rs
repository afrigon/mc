use std::fmt;
use std::str::FromStr;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LoaderKind {
    Fabric
}

impl FromStr for LoaderKind {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "fabric" => Ok(LoaderKind::Fabric),
            _ => anyhow::bail!("loader must be fabric")
        }
    }
}

impl fmt::Display for LoaderKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            LoaderKind::Fabric => "fabric"
        };

        write!(f, "{}", s)
    }
}
