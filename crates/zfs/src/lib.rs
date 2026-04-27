#![warn(clippy::pedantic)]
#![allow(clippy::missing_errors_doc)]

use std::{collections::HashMap, marker::PhantomData};

use serde::Deserialize;

use crate::cli::{FromZfs, ZfsCli, ZfsScalar};
use crate::error::{Error, Result};

pub mod cli;

pub mod error;

pub mod new;

#[derive(Clone, Debug)]
pub struct Property<T> {
    pub value: T,
    pub source: PropertySource,
}

impl<'de, T> Deserialize<'de> for Property<T>
where
    T: FromZfs,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct RawProperty {
            value: ZfsScalar,
            source: PropertySource,
        }

        let raw = RawProperty::deserialize(deserializer)?;

        Ok(Property {
            value: T::from_scalar(raw.value).map_err(serde::de::Error::custom)?,
            source: raw.source,
        })
    }
}

#[derive(Clone, Deserialize, Debug)]
#[serde(rename_all = "UPPERCASE", tag = "type")]
pub enum PropertySource {
    None,
    Default,
    Local,
    Inherited {
        #[serde(rename = "data")]
        from: String,
    },
    #[serde(untagged)]
    Unknown {
        data: String,
    },
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "UPPERCASE")]
pub enum Health {
    Online,
    Degraded,
    Offline,
}

impl FromZfs for Health {
    fn from_scalar(v: ZfsScalar) -> Result<Self> {
        match v {
            ZfsScalar::String(s) => match s.as_str() {
                "ONLINE" => Ok(Health::Online),
                "DEGRADED" => Ok(Health::Degraded),
                "OFFLINE" => Ok(Health::Offline),
                other => Err(Error::UnknownValue {
                    what: "health",
                    value: other.to_string(),
                }),
            },
            _ => Err(Error::InvalidProperty {
                property: "health",
                src: "expected string",
            }),
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct Pool {
    pub state: String,
    pub pool_guid: u64,
    pub txg: u64,
    pub spa_version: u64,
    pub zpl_version: u64,
}

pub type ZPool<B = ZfsCli> = ZfsObject<B, Pool>;

#[derive(Deserialize, Debug)]
pub struct Dataset {}

pub type ZDataset<B = ZfsCli> = ZfsObject<B, Dataset>;

#[derive(Deserialize, Debug)]
pub struct ZfsObject<B, T> {
    pub name: String,
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(flatten)]
    pub inner: T,
    pub properties: HashMap<String, Property<ZfsScalar>>,

    #[serde(skip)]
    _p: PhantomData<B>,
}

#[allow(async_fn_in_trait)]
pub trait ZfsBackend
where
    Self: Sized,
{
    async fn pools() -> Result<HashMap<String, ZPool<Self>>>;

    async fn get_pool(name: &str) -> Result<Option<ZPool<Self>>> {
        Self::get_pool_with_props(name, &[]).await
    }
    async fn get_pool_with_prop(name: &str, prop: &str) -> Result<Option<ZPool<Self>>> {
        Self::get_pool_with_props(name, &[prop]).await
    }
    async fn get_pool_with_props(name: &str, props: &[&str]) -> Result<Option<ZPool<Self>>>;

    async fn datasets() -> Result<HashMap<String, ZDataset<Self>>>;

    async fn get_dataset(name: &str) -> Result<Option<ZDataset<Self>>> {
        Self::get_dataset_with_props(name, &[]).await
    }
    async fn get_dataset_with_prop(name: &str, prop: &str) -> Result<Option<ZDataset<Self>>> {
        Self::get_dataset_with_props(name, &[prop]).await
    }
    async fn get_dataset_with_props(name: &str, props: &[&str]) -> Result<Option<ZDataset<Self>>>;
}

// pub struct Zfs<B: ZfsBackend> {
//     // jh: JoinHandle<()>,
//     cached_pools: HashMap<u64, ZPool<B>>,
// }

// impl Zfs<ZfsCli> {
//     pub fn new() -> Self {
//         // let jh = tokio::spawn(async move { ZfsCli::watch() });

//         Self {
//             // jh,
//             cached_pools: HashMap::default(),
//         }
//     }

//     pub async fn pool(&mut self, name: &str) {
//         if let Some(pool) = self.cached_pools.iter_mut().find(|(_, p)| p.name.eq(name)) {
//         } else {
//         }

//         todo!();
//     }
// }
