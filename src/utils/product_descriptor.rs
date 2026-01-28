use std::fmt;
use std::str::FromStr;

use serde::Deserialize;

use crate::context::McContext;
use crate::utils::errors::McResult;

/// A raw product descriptor is an object that describe a product at a given version. The version might be an alias or not exist at all
#[derive(Clone, Debug)]
pub struct RawProductDescriptor {
    pub product: String,
    pub version: Option<String>
}

impl FromStr for RawProductDescriptor {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();

        if s.is_empty() {
            anyhow::bail!("product descriptor cannot be an empty string")
        };

        let (product, version) = match s.split_once("@") {
            Some((v, n)) => (v.trim(), Some(n.trim())),
            None => (s, None)
        };

        Ok(RawProductDescriptor {
            product: product.to_owned(),
            version: version.map(|v| v.to_owned())
        })
    }
}

impl<'de> Deserialize<'de> for RawProductDescriptor {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>
    {
        let s = String::deserialize(deserializer)?;

        s.parse().map_err(serde::de::Error::custom)
    }
}

/// A product descriptor is an object that describe a product at a given version. The version in this case has been resolved and validated.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ProductDescriptor<P = String, V = String>
where
    P: FromStr + fmt::Display,
    V: FromStr + fmt::Display
{
    pub product: P,
    pub version: V
}

impl<P, V> fmt::Display for ProductDescriptor<P, V>
where
    P: FromStr + fmt::Display,
    V: FromStr + fmt::Display
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}@{}", self.product, self.version)
    }
}

pub trait VersionResolver<P = String, V = String>
where
    P: FromStr + fmt::Display,
    V: FromStr + fmt::Display,
    P::Err: Into<anyhow::Error>,
    V::Err: Into<anyhow::Error>
{
    async fn resolve(context: &McContext, version: Option<String>) -> McResult<V>;

    async fn resolve_descriptor(
        context: &McContext,
        descriptor: RawProductDescriptor
    ) -> McResult<ProductDescriptor<P, V>> {
        Ok(ProductDescriptor {
            product: descriptor.product.parse().map_err(Into::into)?,
            version: Self::resolve(context, descriptor.version).await?
        })
    }
}

// TODO: implement Display
// TODO: investigate a way to reuse the resolver request
