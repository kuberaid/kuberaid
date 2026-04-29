const NAMESPACE: &str = "manager.kuberaid.eu";

const DRIVER_NAME: &str = "zfs.csi.kuberaid.eu";
const DRIVER_VERSION: &str = "v0.0.0";

pub mod crds;
pub mod csi;
pub mod manager;
