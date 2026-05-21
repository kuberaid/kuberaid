use k8s_csi::v1::{
    NodeExpandVolumeRequest, NodeExpandVolumeResponse, NodeGetCapabilitiesRequest,
    NodeGetCapabilitiesResponse, NodeGetInfoRequest, NodeGetInfoResponse,
    NodeGetVolumeStatsRequest, NodeGetVolumeStatsResponse, NodePublishVolumeRequest,
    NodePublishVolumeResponse, NodeStageVolumeRequest, NodeStageVolumeResponse,
    NodeUnpublishVolumeRequest, NodeUnpublishVolumeResponse, NodeUnstageVolumeRequest,
    NodeUnstageVolumeResponse, node_server::Node, node_service_capability,
};
use tonic::*;

use crate::csi::CSIPlugin;

#[tonic::async_trait]
impl Node for CSIPlugin {
    async fn node_stage_volume(
        &self,
        req: Request<NodeStageVolumeRequest>,
    ) -> Result<Response<NodeStageVolumeResponse>, Status> {
        Err(Status::unimplemented(""))
    }

    async fn node_unstage_volume(
        &self,
        req: Request<NodeUnstageVolumeRequest>,
    ) -> Result<Response<NodeUnstageVolumeResponse>, Status> {
        Err(Status::unimplemented(""))
    }

    async fn node_publish_volume(
        &self,
        req: Request<NodePublishVolumeRequest>,
    ) -> Result<Response<NodePublishVolumeResponse>, Status> {
        Err(Status::unimplemented(""))
    }

    async fn node_unpublish_volume(
        &self,
        req: Request<NodeUnpublishVolumeRequest>,
    ) -> Result<Response<NodeUnpublishVolumeResponse>, Status> {
        Err(Status::unimplemented(""))
    }

    async fn node_get_volume_stats(
        &self,
        req: Request<NodeGetVolumeStatsRequest>,
    ) -> Result<Response<NodeGetVolumeStatsResponse>, Status> {
        Err(Status::unimplemented(""))
    }

    async fn node_expand_volume(
        &self,
        req: Request<NodeExpandVolumeRequest>,
    ) -> Result<Response<NodeExpandVolumeResponse>, Status> {
        Err(Status::unimplemented(""))
    }

    async fn node_get_capabilities(
        &self,
        req: Request<NodeGetCapabilitiesRequest>,
    ) -> Result<Response<NodeGetCapabilitiesResponse>, Status> {
        Ok(Response::new(NodeGetCapabilitiesResponse {
            capabilities: vec![node_service_capability::rpc::Type::StageUnstageVolume.into()],
        }))
    }
    async fn node_get_info(
        &self,
        req: Request<NodeGetInfoRequest>,
    ) -> Result<Response<NodeGetInfoResponse>, Status> {
        Ok(Response::new(NodeGetInfoResponse {
            node_id: self.node_name.clone(),
            max_volumes_per_node: i64::MAX,
            accessible_topology: None,
        }))
    }
}
