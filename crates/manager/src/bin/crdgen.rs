use kube::CustomResourceExt;

use kuberaid_manager::crds;

fn main() {
    let crds = [crds::StorageNode::crd(), crds::Pool::crd()];

    print!(
        "---\n{}",
        crds.map(|c| serde_yaml::to_string(&c).unwrap())
            .join("---\n")
    );
}
