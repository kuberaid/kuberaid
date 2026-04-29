use std::{collections::HashMap, str::FromStr};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::{ZDataset, ZPool};

#[derive(Clone, Serialize, Debug, JsonSchema)]
pub enum ZfsScalar {
    U64(u64),
    F32(f32),
    String(String),
    None,
}

impl FromStr for ZfsScalar {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "-" => Ok(ZfsScalar::None),
            s if s.starts_with("0x") => {
                if s.contains(' ') {
                    Ok(ZfsScalar::String(s.to_owned()))
                } else {
                    Ok(ZfsScalar::U64(u64::from_str_radix(
                        s.trim_start_matches("0x"),
                        16,
                    )?))
                }
            }
            _ => {
                if let Ok(p) = s.parse::<u64>() {
                    Ok(ZfsScalar::U64(p))
                } else if let Ok(p) = s.parse::<f32>() {
                    Ok(ZfsScalar::F32(p))
                } else {
                    Ok(ZfsScalar::String(s.to_owned()))
                }
            }
        }
    }
}

impl<'de> Deserialize<'de> for ZfsScalar {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Raw {
            U64(u64),
            F32(f32),
            String(String),
        }

        match Raw::deserialize(deserializer)? {
            Raw::U64(v) => Ok(ZfsScalar::U64(v)),
            Raw::F32(v) => Ok(ZfsScalar::F32(v)),
            Raw::String(s) if s == "-" => Ok(ZfsScalar::None),
            Raw::String(s) => Ok(ZfsScalar::String(s)),
        }
    }
}

pub trait FromZfs: Sized {
    fn from_scalar(v: ZfsScalar) -> Result<Self>;
}

impl FromZfs for u64 {
    fn from_scalar(v: ZfsScalar) -> Result<Self> {
        match v {
            ZfsScalar::U64(x) => Ok(x),
            ZfsScalar::String(s) => s.parse().map_err(Error::ParseInt),
            _ => Err(Error::InvalidProperty {
                property: "u64",
                src: "expected u64 or string representation",
            }),
        }
    }
}

impl FromZfs for f32 {
    fn from_scalar(v: ZfsScalar) -> Result<Self> {
        match v {
            ZfsScalar::F32(x) => Ok(x),
            ZfsScalar::U64(x) => Ok(x as f32),
            ZfsScalar::String(s) => s.parse().map_err(Error::ParseFloat),
            _ => Err(Error::InvalidProperty {
                property: "f32",
                src: "expected f32, u64, or string representation",
            }),
        }
    }
}

impl FromZfs for String {
    fn from_scalar(v: ZfsScalar) -> Result<Self> {
        match v {
            ZfsScalar::String(s) => Ok(s),
            ZfsScalar::U64(n) => Ok(n.to_string()),
            ZfsScalar::F32(f) => Ok(f.to_string()),
            ZfsScalar::None => Ok("-".to_string()),
        }
    }
}

impl FromZfs for bool {
    fn from_scalar(v: ZfsScalar) -> Result<Self> {
        match v {
            ZfsScalar::String(s) if s.eq("off") => Ok(false),
            ZfsScalar::String(s) if s.eq("on") => Ok(true),
            ZfsScalar::String(s) if s.eq("disabled") => Ok(false),
            ZfsScalar::String(s) if s.eq("enabled") => Ok(true),
            ZfsScalar::String(s) if s.eq("inactive") => Ok(false),
            ZfsScalar::String(s) if s.eq("active") => Ok(true),
            ZfsScalar::U64(n) => Ok(n != 0),
            ZfsScalar::F32(f) => Ok(f != 0f32),
            ZfsScalar::None => Ok(false),
            ZfsScalar::String(s) => Err(Error::UnknownValue {
                what: "boolean",
                value: s,
            }),
        }
    }
}

impl<T> FromZfs for Option<T>
where
    T: FromZfs,
{
    fn from_scalar(v: ZfsScalar) -> Result<Self> {
        match v {
            ZfsScalar::None => Ok(None),
            other => T::from_scalar(other).map(Some),
        }
    }
}

impl FromZfs for ZfsScalar {
    fn from_scalar(v: ZfsScalar) -> Result<Self> {
        Ok(v)
    }
}

#[derive(Deserialize, Debug)]
pub struct Pools {
    pub pools: HashMap<String, ZPool>,
}

#[derive(Deserialize, Debug)]
pub struct Datasets {
    pub datasets: HashMap<String, ZDataset>,
}

#[derive(Deserialize)]
pub struct ZfsOutput<T> {
    output_version: OutputVersion,
    #[serde(flatten)]
    pub inner: T,
}

#[derive(Deserialize)]
pub struct OutputVersion {
    command: String,
    vers_major: u8,
    vers_minor: u8,
}
