#![warn(clippy::pedantic)]

pub mod crds;
pub mod grpc {
    tonic::include_proto!("kuberaid.v1");
}
