use std::{collections::HashMap, pin::Pin, str::FromStr};

use chrono::{DateTime, NaiveDateTime, Utc};
use itertools::Itertools;

use serde::Deserialize;
use tokio::{io::BufReader, process::Command};
use tracing::{error, info};

use crate::error::{Error, Result};
use crate::{
    ZDataset, ZPool, ZfsBackend,
    cli::model::{Datasets, Pools, ZfsOutput},
};
use futures::{Stream, StreamExt};
use tokio_util::codec::{AnyDelimiterCodec, FramedRead, LinesCodec};

mod model;

pub use model::{FromZfs, ZfsScalar};

#[derive(Debug)]
pub struct ZfsCli {}

async fn call<T: serde::de::DeserializeOwned>(cmd: &mut Command) -> Result<Option<T>> {
    let output = cmd.args(["-j", "--json-int"]).output().await?;

    if output.status.success() {
        if output.stdout.is_empty() {
            Ok(None)
        } else {
            Ok(Some(serde_json::from_slice(&output.stdout)?))
        }
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);

        Err(Error::CommandFailed {
            code: output.status.code(),
            stdout: stdout.to_string(),
            stderr: stderr.to_string(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct DatasetRef {
    pub id: u64,
    pub name: String,
}

#[derive(Debug, Clone)]
pub enum HistoryEventKind {
    Set {
        property: String,
        value: ZfsScalar,

        dataset: Option<DatasetRef>,
    },
    Import,
    Open,

    Unknown {
        name: String,
        value: String,
        dataset: Option<DatasetRef>,
    },
}

impl TryFrom<&mut RawProps> for HistoryEventKind {
    type Error = Error;

    fn try_from(p: &mut RawProps) -> Result<Self> {
        let s = get_key::<String>(p, "history_internal_name")?;
        let internal = get_key::<String>(p, "history_internal_str")?;
        let dataset = if p.contains_key("history_dsid") && p.contains_key("histroy_dsname") {
            Some(DatasetRef {
                id: get_key(p, "history_dsid")?,
                name: get_key(p, "history_dsname")?,
            })
        } else {
            None
        };

        match s.as_str() {
            "set" => {
                let (k, v) = internal.split_once('=').unwrap();

                Ok(Self::Set {
                    property: k.to_string(),
                    value: ZfsScalar::from_str(v)?,
                    dataset,
                })
            }

            _ => Ok(Self::Unknown {
                name: s,
                value: internal,
                dataset,
            }),
        }
    }
}

// impl FromZfs for HistoryEventKind {
//     fn from_scalar(v: ZfsScalar) -> Result<Self> {
//         match v {
//             ZfsScalar::String(s) => match s.as_str() {
//                 "set" => Ok(HistoryEventKind::Set),
//                 "import" => Ok(HistoryEventKind::Import),
//                 "open" => Ok(HistoryEventKind::Open),
//                 _ => Err(Error::UnknownValue {
//                     what: "history_internal_name",
//                     value: s,
//                 }),
//             },
//             _ => Err(Error::InvalidProperty {
//                 property: "history_internal_name",
//                 src: "expected string",
//             }),
//         }
//     }
// }

#[derive(Debug, Clone)]
pub enum ZfsEventKind {
    HistoryEvent {
        hostname: String,
        txg: u64,
        kind: HistoryEventKind,
    },
    ConfigSync,
    PoolImport,
    PoolExport,
    PoolDestroy,
    Raw(RawProps),
}

#[derive(Debug, Clone)]
pub struct ZfsEvent<T = ZfsEventKind> {
    pub class: String,
    pub version: u64,
    pub eid: u64,
    pub pool: String,
    pub pool_state: u64,
    pub pool_context: u64,
    pub pool_guid: u64,
    pub kind: T,
    pub time: chrono::DateTime<Utc>,
}

type RawProps = HashMap<String, ZfsScalar>;
type RawZfsEvent = ZfsEvent<RawProps>;

pub fn get_key<T: FromZfs>(v: &mut RawProps, key: &str) -> Result<T> {
    T::from_scalar(v.remove(key).ok_or(Error::MissingKey(key.to_string()))?)
}

impl TryFrom<RawZfsEvent> for ZfsEvent {
    type Error = Error;

    fn try_from(mut v: RawZfsEvent) -> Result<Self> {
        let kind = match v.class.as_str() {
            "sysevent.fs.zfs.history_event" => ZfsEventKind::HistoryEvent {
                hostname: get_key(&mut v.kind, "history_hostname")?,
                // kind: get_key(&mut v.kind, "history_internal_name")?,
                kind: HistoryEventKind::try_from(&mut v.kind)?,
                txg: get_key(&mut v.kind, "history_txg")?,
            },
            "sysevent.fs.zfs.config_sync" => ZfsEventKind::ConfigSync,
            "sysevent.fs.zfs.pool_import" => ZfsEventKind::PoolImport,
            "sysevent.fs.zfs.pool_export" => ZfsEventKind::PoolExport,
            "sysevent.fs.zfs.pool_destroy" => ZfsEventKind::PoolDestroy,
            _ => ZfsEventKind::Raw(v.kind),
        };

        Ok(ZfsEvent {
            kind,
            class: v.class,
            version: v.version,
            eid: v.eid,
            pool: v.pool,
            pool_state: v.pool_state,
            pool_guid: v.pool_guid,
            pool_context: v.pool_context,
            time: v.time,
        })
    }
}

fn parse_time(str: &str) -> Result<chrono::DateTime<Utc>> {
    let (secs, nanos) = str.split_once(' ').ok_or_else(|| {
        Error::EventParse(format!(
            "invalid time format: expected 'secs nanos', got '{str}'"
        ))
    })?;
    let secs = i64::from_str_radix(secs.trim_start_matches("0x"), 16)
        .map_err(|_| Error::EventParse(format!("invalid seconds hex in: {str}")))?;
    let nanos = u32::from_str_radix(nanos.trim_start_matches("0x"), 16)
        .map_err(|_| Error::EventParse(format!("invalid nanoseconds hex in: {str}")))?;

    DateTime::from_timestamp(secs, nanos)
        .ok_or_else(|| Error::EventParse(format!("timestamp out of range: {secs}")))
}

impl FromStr for RawZfsEvent {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let mut props = HashMap::default();

        let mut lines = s.lines();
        let class = lines.next().unwrap();

        let mut time = None;
        while let Some(line) = lines.next()
            && !line.is_empty()
        {
            let (k, v) = line.trim().split_once('=').unwrap();
            let k = k.trim().trim_matches(['"']);
            let v = v.trim().trim_matches(['"']);

            if k.eq("time") {
                time = Some(parse_time(v)?);
            } else {
                props.insert(
                    k.to_string(),
                    ZfsScalar::from_str(v.trim()).map_err(|e| Error::EventParse(e.to_string()))?,
                );
            }
        }

        if time.is_none() && !props.contains_key("class") {
            Err(Error::EventParse(format!("not an event: {s}")))
        } else {
            let prop_class = get_key::<String>(&mut props, "class")?;
            debug_assert_eq!(class, prop_class);

            let version = get_key(&mut props, "version")?;
            let eid = get_key(&mut props, "eid")?;
            let pool = get_key(&mut props, "pool")?;
            let pool_guid = get_key(&mut props, "pool_guid")?;
            let pool_state = get_key(&mut props, "pool_state")?;

            let pool_context = get_key(&mut props, "pool_context")?;

            Ok(Self {
                class: class.to_string(),
                kind: props,
                time: time.unwrap_or_default(),
                version,
                eid,
                pool,
                pool_state,
                pool_guid,
                pool_context,
            })
        }
    }
}

pub type EventStream = Pin<Box<dyn Stream<Item = ZfsEvent> + Send>>;

impl ZfsCli {
    fn zfs() -> Command {
        Command::new("zfs")
    }

    fn zpool() -> Command {
        Command::new("zpool")
    }

    pub fn events() -> Result<EventStream> {
        let mut cmd = Self::zpool()
            .args(["events", "-vHf"])
            .stdout(std::process::Stdio::piped())
            .spawn()?;
        let stdout = cmd.stdout.take().ok_or(Error::EventStreamClosed)?;

        let reader = BufReader::new(stdout);
        let blocks = FramedRead::new(
            reader,
            AnyDelimiterCodec::new(b"\t".to_vec(), b"\t".to_vec()),
        );

        let stream = blocks
            .map(|line_result| {
                let line = line_result.map_err(|e| Error::EventParse(format!("{e}")))?;
                let string = String::from_utf8_lossy(&line).to_string();

                RawZfsEvent::from_str(&string)
            })
            .map(|e| {
                #[cfg(debug_assertions)]
                let str_e = format!("{e:?}");
                let r = e.and_then(ZfsEvent::try_from);
                #[cfg(debug_assertions)]
                let r = r.inspect_err(|r| error!("{r}: {str_e}"));
                r
            })
            .filter_map(async |s| s.ok());

        Ok(Box::pin(stream))
    }
}

impl ZfsBackend for ZfsCli {
    async fn pools() -> Result<HashMap<String, ZPool<Self>>> {
        let out = call::<ZfsOutput<Pools>>(ZfsCli::zpool().args(["list", "--json-pool-key-guid"]))
            .await?;

        Ok(out.map(|o| o.inner.pools).unwrap_or_default())
    }

    async fn get_pool_with_props(name: &str, props: &[&str]) -> Result<Option<ZPool<Self>>> {
        let props = props.iter().filter(|s| !s.is_empty()).join(",");
        let out = call::<ZfsOutput<Pools>>(ZfsCli::zpool().args([
            "get",
            if props.is_empty() { "all" } else { &props },
            name,
            // "--json-pool-key-guid",
        ]))
        .await?;
        Ok(out.and_then(|mut o| o.inner.pools.remove(name)))
    }

    async fn datasets() -> Result<HashMap<String, ZDataset<Self>>> {
        let out = call::<ZfsOutput<Datasets>>(ZfsCli::zfs().args(["list"])).await?;

        Ok(out.map(|o| o.inner.datasets).unwrap_or_default())
    }

    async fn get_dataset_with_props(name: &str, props: &[&str]) -> Result<Option<ZDataset<Self>>> {
        let props = props.iter().filter(|s| !s.is_empty()).join(",");
        let out = call::<ZfsOutput<Datasets>>(ZfsCli::zfs().args([
            "get",
            if props.is_empty() { "all" } else { &props },
            name,
        ]))
        .await?;

        Ok(out.and_then(|mut o| o.inner.datasets.remove(name)))
    }
}

impl ZDataset<ZfsCli> {
    pub fn property<T: FromZfs>(&self, name: &str) -> Result<Option<T>> {
        let value = match self.properties.get(name) {
            Some(p) => p.value.clone(),
            None => return Ok(None),
        };

        T::from_scalar(value).map(Some)
    }

    pub async fn refresh_property<T: FromZfs>(&mut self, name: &str) -> Result<Option<T>> {
        let fetched = ZfsCli::get_dataset_with_prop(&self.name, name).await?;
        let prop = fetched.and_then(|mut d| d.properties.remove(name));

        if let Some(prop) = prop {
            let v = prop.value.clone();
            self.properties.insert(name.to_string(), prop);

            T::from_scalar(v).map(Some)
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {

    use futures::StreamExt;

    use crate::ZfsBackend;
    use crate::error::Result;

    use super::ZfsCli;

    #[tokio::test]
    pub async fn it_works() -> Result<()> {
        let pools = ZfsCli::pools().await?;

        assert_eq!(pools.len(), 1);
        assert_eq!(pools.iter().next().unwrap().1.name, "lab");

        eprintln!("{pools:?}");

        let mut datasets = ZfsCli::datasets().await?;
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

        let mut events = ZfsCli::events()?;
        while let Some(event) = events.next().await {
            println!("event: {event:?}");
        }

        Ok(())
    }
}
