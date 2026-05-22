use std::time::Duration;

use zfs::ZfsBackend;
use zfs::cli::ZfsCli;
use zfs::new::Zfs;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
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

    // let mut events = ZfsCli::events()?;
    // while let Some(event) = events.next().await {
    //     println!("event: {event:?}");
    // }

    let zfs = Zfs::new().await?;

    loop {
        let dataset = zfs.get_dataset("lab").await.unwrap();
        println!("{dataset:?}");
        tokio::time::sleep(Duration::from_secs(1)).await;
    }

    Ok(())
}
