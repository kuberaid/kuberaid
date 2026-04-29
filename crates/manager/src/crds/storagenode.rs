use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(CustomResource, Deserialize, Serialize, Clone, Debug, JsonSchema, Default)]
#[kube(kind = "StorageNode", group = "kuberaid.eu", version = "v1")]
#[kube(status = "StorageNodeStatus", shortname = "sn")]
#[serde(rename_all = "camelCase")]
pub struct StorageNodeSpec {
    // pub zfs: ZfsStorage,
}

#[derive(Deserialize, Serialize, Clone, Default, Debug, JsonSchema)]
pub struct ZfsStorage {}

// #[derive(Deserialize, Serialize, Clone, Default, Debug, JsonSchema)]
// pub struct StorageDevice {
//     pub name: String,
// }

/// The status object of `StorageNode`
#[derive(Deserialize, Serialize, Clone, Default, Debug, JsonSchema)]
pub struct StorageNodeStatus {
    pub zfs: Option<ZfsStorageStatus>,
}

#[derive(Deserialize, Serialize, Clone, Default, Debug, JsonSchema)]
pub struct ZfsStorageStatus {
    // pub host_id: String,
}
