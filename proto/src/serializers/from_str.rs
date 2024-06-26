//! Serialize and deserialize any `T` that implements [[core::str::FromStr]]
//! and [[core::fmt::Display]] from or into string. Note this can be used for
//! all primitive data types.
#[cfg(not(feature = "grpc"))]
use core::{fmt::Display, str::FromStr};
#[cfg(feature = "grpc")]
use std::{fmt::Display, str::FromStr};

use serde::{de::Error as _, Deserialize, Deserializer, Serialize, Serializer};

use crate::prelude::*;
/// Deserialize string into T
pub fn deserialize<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: FromStr,
    <T as FromStr>::Err: Display,
{
    String::deserialize(deserializer)?
        .parse::<T>()
        .map_err(|e| D::Error::custom(format!("{e}")))
}

/// Serialize from T into string
pub fn serialize<S, T>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    T: Display,
{
    format!("{value}").serialize(serializer)
}
