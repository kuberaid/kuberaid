use kube::CustomResourceExt;

use kuberaid_common::crds;

fn main() {
    print!(
        "{}",
        serde_yaml::to_string(&crds::StorageNode::crd()).unwrap()
    )
}
