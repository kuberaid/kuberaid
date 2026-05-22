use k8s_csi::{
    self as csi,
    v1::{node_server::Node, node_service_capability as nsc},
};
use tonic::*;

use crate::csi::CSIPlugin;

#[tonic::async_trait]
impl Node for CSIPlugin {
    async fn node_stage_volume(
        &self,
        req: Request<csi::v1::NodeStageVolumeRequest>,
    ) -> Result<Response<csi::v1::NodeStageVolumeResponse>, Status> {
        Err(Status::unimplemented(""))
    }

    async fn node_unstage_volume(
        &self,
        req: Request<csi::v1::NodeUnstageVolumeRequest>,
    ) -> Result<Response<csi::v1::NodeUnstageVolumeResponse>, Status> {
        Err(Status::unimplemented(""))
    }

    async fn node_publish_volume(
        &self,
        req: Request<csi::v1::NodePublishVolumeRequest>,
    ) -> Result<Response<csi::v1::NodePublishVolumeResponse>, Status> {
        Err(Status::unimplemented(""))
    }

    async fn node_unpublish_volume(
        &self,
        req: Request<csi::v1::NodeUnpublishVolumeRequest>,
    ) -> Result<Response<csi::v1::NodeUnpublishVolumeResponse>, Status> {
        Err(Status::unimplemented(""))
    }

    async fn node_get_volume_stats(
        &self,
        req: Request<csi::v1::NodeGetVolumeStatsRequest>,
    ) -> Result<Response<csi::v1::NodeGetVolumeStatsResponse>, Status> {
        Err(Status::unimplemented(""))
    }

    async fn node_expand_volume(
        &self,
        req: Request<csi::v1::NodeExpandVolumeRequest>,
    ) -> Result<Response<csi::v1::NodeExpandVolumeResponse>, Status> {
        Err(Status::unimplemented(""))
    }

    async fn node_get_capabilities(
        &self,
        req: Request<csi::v1::NodeGetCapabilitiesRequest>,
    ) -> Result<Response<csi::v1::NodeGetCapabilitiesResponse>, Status> {
        Ok(Response::new(csi::v1::NodeGetCapabilitiesResponse {
            capabilities: vec![
                nsc::rpc::Type::StageUnstageVolume.into(),
                nsc::rpc::Type::ExpandVolume.into(),
            ],
        }))
    }
    async fn node_get_info(
        &self,
        req: Request<csi::v1::NodeGetInfoRequest>,
    ) -> Result<Response<csi::v1::NodeGetInfoResponse>, Status> {
        Ok(Response::new(csi::v1::NodeGetInfoResponse {
            node_id: self.node_name.clone(),
            max_volumes_per_node: i64::MAX,
            accessible_topology: None,
        }))
    }
}
