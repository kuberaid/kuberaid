#![warn(clippy::pedantic)]

use std::{collections::HashMap, os::fd::AsFd};

use anyhow::Result;
use itertools::Itertools;
use serde::{Deserialize, Deserializer};
use tokio::{
    io::Stdout,
    process::{ChildStdout, Command},
};

#[derive(Deserialize)]
struct OutputVersion {
    command: String,
    vers_major: u8,
    vers_minor: u8,
}

#[derive(Clone, Debug)]
pub enum ZfsScalar {
    U64(u64),
    F64(f64),
    String(String),
    None,
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
            F64(f64),
            String(String),
        }

        match Raw::deserialize(deserializer)? {
            Raw::U64(v) => Ok(ZfsScalar::U64(v)),
            Raw::F64(v) => Ok(ZfsScalar::F64(v)),
            Raw::String(s) if s == "-" => Ok(ZfsScalar::None),
            Raw::String(s) => Ok(ZfsScalar::String(s)),
        }
    }
}

pub trait FromZfs: Sized {
    fn from_scalar(v: ZfsScalar) -> Result<Self, String>;
}

impl FromZfs for u64 {
    fn from_scalar(v: ZfsScalar) -> Result<Self, String> {
        match v {
            ZfsScalar::U64(x) => Ok(x),
            ZfsScalar::String(s) => s
                .parse()
                .map_err(|e: std::num::ParseIntError| e.to_string()),
            _ => Err("invalid u64".into()),
        }
    }
}

impl FromZfs for f64 {
    fn from_scalar(v: ZfsScalar) -> Result<Self, String> {
        match v {
            ZfsScalar::F64(x) => Ok(x),
            ZfsScalar::U64(x) => Ok(x as f64),
            ZfsScalar::String(s) => s
                .parse()
                .map_err(|e: std::num::ParseFloatError| e.to_string()),
            _ => Err("invalid f64".into()),
        }
    }
}

impl FromZfs for String {
    fn from_scalar(v: ZfsScalar) -> Result<Self, String> {
        match v {
            ZfsScalar::String(s) => Ok(s),
            ZfsScalar::U64(n) => Ok(n.to_string()),
            ZfsScalar::F64(f) => Ok(f.to_string()),
            ZfsScalar::None => Ok("-".to_string()),
        }
    }
}

impl FromZfs for bool {
    fn from_scalar(v: ZfsScalar) -> Result<Self, String> {
        match v {
            ZfsScalar::String(s) if s.eq("off") => Ok(false),
            ZfsScalar::String(s) if s.eq("on") => Ok(true),
            ZfsScalar::U64(n) => Ok(n != 0),
            ZfsScalar::F64(f) => Ok(f != 0f64),
            ZfsScalar::None => Ok(false),
            ZfsScalar::String(_) => todo!(),
        }
    }
}

impl FromZfs for ZfsScalar {
    fn from_scalar(v: ZfsScalar) -> Result<Self, String> {
        Ok(v)
    }
}

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

#[derive(Debug)]
pub struct FloatFromStr(pub f64);

impl<'de> Deserialize<'de> for FloatFromStr {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum FloatOrString {
            Float(f64),
            String(String),
        }

        match FloatOrString::deserialize(deserializer)? {
            FloatOrString::Float(f) => Ok(FloatFromStr(f)),
            FloatOrString::String(s) => s
                .parse()
                .map(FloatFromStr)
                .map_err(serde::de::Error::custom),
        }
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "UPPERCASE")]
pub enum Health {
    Online,
    Degraded,
    Offline,
}

impl FromZfs for Health {
    fn from_scalar(v: ZfsScalar) -> Result<Self, String> {
        match v {
            ZfsScalar::String(s) => match s.as_str() {
                "ONLINE" => Ok(Health::Online),
                "DEGRADED" => Ok(Health::Degraded),
                "OFFLINE" => Ok(Health::Offline),
                other => Err(format!("unknown health state: {other}")),
            },
            _ => Err("health must be string".into()),
        }
    }
}

impl<T> FromZfs for Option<T>
where
    T: FromZfs,
{
    fn from_scalar(v: ZfsScalar) -> Result<Self, String> {
        match v {
            ZfsScalar::None => Ok(None),
            other => T::from_scalar(other).map(Some),
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct PoolProperties {
    pub size: Property<u64>,
    pub allocated: Property<u64>,
    pub free: Property<u64>,

    pub fragmentation: Property<u64>,
    pub capacity: Property<u64>,

    pub dedupratio: Property<Option<f64>>,
    pub health: Property<Health>,

    pub altroot: Property<Option<String>>,

    // optional-ish weird ones
    pub checkpoint: Property<Option<u64>>,
    pub expandsize: Property<Option<u64>>,

    #[serde(flatten)]
    pub extra: HashMap<String, Property<ZfsScalar>>,
}

#[derive(Deserialize, Debug)]
pub struct Pool {
    pub state: String,
    pub pool_guid: u64,
    pub txg: u64,
    pub spa_version: u64,
    pub zpl_version: u64,
}

pub type ZPool = ZfsObject<Pool, PoolProperties>;

#[derive(Deserialize, Debug)]
pub struct Dataset {}

pub type ZDataset = ZfsObject<Dataset>;

#[derive(Deserialize, Debug)]
pub struct ZfsProperties<T> {
    #[serde(flatten)]
    pub inner: T,
    #[serde(flatten)]
    pub extra: HashMap<String, Property<ZfsScalar>>,
}

#[derive(Deserialize, Debug)]
pub struct ZfsObject<T, P = ()> {
    pub name: String,
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(flatten)]
    pub inner: T,
    pub properties: ZfsProperties<P>,
}

#[derive(Deserialize, Debug)]
struct Pools {
    pools: HashMap<String, ZPool>,
}

#[derive(Deserialize, Debug)]
struct Datasets {
    datasets: HashMap<String, ZDataset>,
}

#[derive(Deserialize)]
struct ZfsOutput<T> {
    output_version: OutputVersion,
    #[serde(flatten)]
    inner: T,
}

pub struct ZfsCli {}

async fn call<T: serde::de::DeserializeOwned>(cmd: &mut Command) -> anyhow::Result<T> {
    let output = cmd.args(["-j", "--json-int"]).output().await?;

    if output.status.success() {
        Ok(serde_json::from_slice(&output.stdout)?)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);

        Err(anyhow::anyhow!(
            "zfs command failed (code: {:?})\nstdout: {}\nstderr: {}",
            output.status.code(),
            stdout,
            stderr
        ))
    }
}

impl ZfsCli {
    fn zfs() -> Command {
        Command::new("zfs")
    }

    fn zpool() -> Command {
        Command::new("zpool")
    }

    pub async fn pools() -> Result<HashMap<String, ZPool>> {
        let out: ZfsOutput<Pools> = call(ZfsCli::zpool().args(["list"])).await?;

        Ok(out.inner.pools)
    }

    pub async fn datasets() -> Result<HashMap<String, ZDataset>> {
        let mut out: ZfsOutput<Datasets> = call(ZfsCli::zfs().args(["list"])).await?;

        Ok(out.inner.datasets)
    }

    pub async fn get_dataset(name: &str) -> Result<Option<ZDataset>> {
        Self::get_dataset_with_props(name, &[]).await
    }

    pub async fn get_dataset_with_prop(name: &str, prop: &str) -> Result<Option<ZDataset>> {
        Self::get_dataset_with_props(name, &[prop]).await
    }

    pub async fn get_dataset_with_props(name: &str, props: &[&str]) -> Result<Option<ZDataset>> {
        let props = props.iter().filter(|s| !s.is_empty()).join(",");
        let mut out: ZfsOutput<Datasets> =
            call(ZfsCli::zfs().args(["get", if props.is_empty() { "all" } else { &props }, name]))
                .await?;

        Ok(out.inner.datasets.remove(name))
    }
}

impl ZDataset {
    #[must_use]
    pub fn property<T: FromZfs>(&self, name: &str) -> Result<Option<T>> {
        let value = match self.properties.extra.get(name) {
            Some(p) => p.value.clone(),
            None => return Ok(None),
        };

        T::from_scalar(value).map(Some).map_err(anyhow::Error::msg)
    }

    pub async fn refresh_property<T: FromZfs>(&mut self, name: &str) -> Result<Option<T>> {
        let fetched = ZfsCli::get_dataset_with_prop(&self.name, name).await?;
        let prop = fetched.and_then(|mut d| d.properties.extra.remove(name));

        if let Some(prop) = prop {
            let v = prop.value.clone();
            self.properties.extra.insert(name.to_string(), prop);

            T::from_scalar(v)
                .map(|v| Some(v))
                .map_err(anyhow::Error::msg)
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {

    #[tokio::test]
    pub async fn it_works() -> anyhow::Result<()> {
        let pools = crate::ZfsCli::pools().await?;

        assert_eq!(pools.len(), 1);
        assert_eq!(pools.iter().next().unwrap().1.name, "lab");

        eprintln!("{pools:?}");

        let mut datasets = crate::ZfsCli::datasets().await?;
        eprintln!("Datasets: {datasets:?}");

        let lab = datasets.get_mut("lab/test").unwrap();

        let atime: Option<bool> = lab.property("atime")?;
        assert!(atime.is_none());
        let _: Option<bool> = lab.refresh_property("atime").await?;
        let atime: Option<bool> = lab.property("atime")?;
        eprintln!("atime: {atime:?}");

        let quota: Option<u64> = lab.property("quota")?;
        assert!(quota.is_none());
        let _: Option<bool> = lab.refresh_property("quota").await?;
        let quota: Option<u64> = lab.property("quota")?;
        eprintln!("quota: {quota:?}");

        Ok(())
    }
}
