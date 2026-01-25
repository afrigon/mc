use serde::Deserialize;
use std::fmt;
use std::str::FromStr;

use crate::{
    java::{JavaVendor, JavaVersion},
    utils::CaseIterable
};

#[derive(Clone, Copy, PartialEq, Eq, Deserialize)]
pub struct JavaDistribution {
    pub vendor: JavaVendor,
    pub version: JavaVersion
}

impl Default for JavaDistribution {
    fn default() -> JavaDistribution {
        JavaDistribution {
            vendor: JavaVendor::graal,
            version: JavaVersion::Java25
        }
    }
}

impl fmt::Display for JavaDistribution {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}@{}", self.vendor, self.version)
    }
}

impl FromStr for JavaDistribution {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();

        if s.is_empty() {
            return Err("invalid java distribution format");
        };

        let (vendor_string, version_string) = match s.split_once("@") {
            Some((v, n)) => (v.trim(), Some(n.trim())),
            None => (s, None)
        };

        let vendor = vendor_string.parse::<JavaVendor>()?;
        let version = match version_string {
            None => vendor.latest_version(),
            Some(v) if v.is_empty() || v.eq_ignore_ascii_case("latest") => vendor.latest_version(),
            Some(v) => v.parse()?
        };

        Ok(JavaDistribution { vendor, version })
    }
}

impl CaseIterable for JavaDistribution {
    fn all_cases() -> &'static [Self] {
        &[
            JavaDistribution {
                vendor: JavaVendor::correto,
                version: JavaVersion::Java25
            },
            JavaDistribution {
                vendor: JavaVendor::correto,
                version: JavaVersion::Java21
            },
            JavaDistribution {
                vendor: JavaVendor::correto,
                version: JavaVersion::Java17
            },
            JavaDistribution {
                vendor: JavaVendor::correto,
                version: JavaVersion::Java11
            },
            JavaDistribution {
                vendor: JavaVendor::correto,
                version: JavaVersion::Java8
            },
            JavaDistribution {
                vendor: JavaVendor::graal,
                version: JavaVersion::Java25
            },
            JavaDistribution {
                vendor: JavaVendor::graal,
                version: JavaVersion::Java21
            },
            JavaDistribution {
                vendor: JavaVendor::graal,
                version: JavaVersion::Java17
            }
        ]
    }
}
