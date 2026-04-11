use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(CustomResource, Deserialize, Serialize, Clone, Debug, JsonSchema)]
#[cfg_attr(test, derive(Default))]
#[kube(kind = "StorageNode", group = "kuberaid.eu", version = "v1")]
#[kube(status = "StorageNodeStatus", shortname = "sn")]
#[serde(rename_all = "camelCase")]
pub struct StorageNodeSpec {
    pub node_id: String,
    pub kube_node_ref: k8s_openapi::api::core::v1::TypedObjectReference,

    pub storage_devices: Vec<StorageDevice>,
}

#[derive(Deserialize, Serialize, Clone, Default, Debug, JsonSchema)]
pub struct StorageDevice {
    pub name: String,
}

/// The status object of `StorageNode`
#[derive(Deserialize, Serialize, Clone, Default, Debug, JsonSchema)]
pub struct StorageNodeStatus {
    pub hidden: bool,
}
