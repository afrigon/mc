use std::fmt;
use std::marker::PhantomData;
use std::str::FromStr;

use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;
use serde::de::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SeparatedList<T, const SEP: char> {
    data: Vec<T>,
    separator: PhantomData<T>
}

impl<T, const SEP: char> Default for SeparatedList<T, SEP> {
    fn default() -> Self {
        SeparatedList {
            data: Vec::new(),
            separator: PhantomData
        }
    }
}

impl<T, const SEP: char> From<Vec<T>> for SeparatedList<T, SEP> {
    fn from(v: Vec<T>) -> Self {
        SeparatedList {
            data: v,
            separator: PhantomData
        }
    }
}

impl<T, const SEP: char> AsRef<[T]> for SeparatedList<T, SEP> {
    fn as_ref(&self) -> &[T] {
        &self.data
    }
}

impl<T, const SEP: char> Serialize for SeparatedList<T, SEP>
where
    T: fmt::Display
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer
    {
        let s = self
            .data
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<_>>()
            .join(&SEP.to_string());

        serializer.serialize_str(&s)
    }
}

impl<'de, T, const SEP: char> Deserialize<'de> for SeparatedList<T, SEP>
where
    T: FromStr,
    T::Err: fmt::Display
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>
    {
        let s = String::deserialize(deserializer)?;

        if s.trim().is_empty() {
            return Ok(SeparatedList {
                data: Vec::new(),
                separator: PhantomData
            });
        }

        let mut data = Vec::new();
        for raw in s.split(SEP) {
            let part = raw.trim();

            if part.is_empty() {
                return Err(D::Error::custom("empty item in separated list"));
            }

            data.push(part.parse::<T>().map_err(D::Error::custom)?);
        }

        Ok(SeparatedList {
            data,
            separator: PhantomData
        })
    }
}
