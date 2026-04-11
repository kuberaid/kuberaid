use kube::CustomResourceExt;

use kuberaid::crds;

fn main() {
    print!(
        "{}",
        serde_yaml::to_string(&crds::StorageNode::crd()).unwrap()
    )
}
