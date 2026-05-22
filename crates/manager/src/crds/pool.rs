use std::collections::HashMap;

use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(CustomResource, Deserialize, Serialize, Clone, Debug, JsonSchema, Default)]
#[kube(kind = "Pool", group = "zfs.kuberaid.eu", version = "v1")]
#[kube(status = "PoolStatus", shortname = "pool")]
#[kube(
    printcolumn(
        name = "Imported",
        type_ = "boolean",
        description = "Is pool imported",
        json_path = ".status.imported"
    ),
    printcolumn(
        name = "Age",
        type_ = "date",
        json_path = ".metadata.creationTimestamp"
    )
)]
#[serde(rename_all = "camelCase")]
pub struct PoolSpec {}

/// The status object of `Pool`
#[derive(Deserialize, Serialize, Clone, Default, Debug, JsonSchema)]
pub struct PoolStatus {
    pub imported: bool,

    pub properties: HashMap<String, String>,
}

impl Default for Pool {
    fn default() -> Self {
        Self {
            metadata: Default::default(),
            spec: Default::default(),
            status: Default::default(),
        }
    }
}
