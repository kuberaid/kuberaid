use k8s_csi::{
    self as csi,
    v1::{controller_server::Controller, controller_service_capability as csc},
};
use tonic::*;

use crate::csi::CSIPlugin;

#[tonic::async_trait]
impl Controller for CSIPlugin {
    async fn create_volume(
        &self,
        req: Request<csi::v1::CreateVolumeRequest>,
    ) -> Result<Response<csi::v1::CreateVolumeResponse>, Status> {
        Err(Status::unimplemented(""))
    }

    async fn delete_volume(
        &self,
        req: Request<csi::v1::DeleteVolumeRequest>,
    ) -> Result<Response<csi::v1::DeleteVolumeResponse>, Status> {
        Err(Status::unimplemented(""))
    }

    async fn controller_publish_volume(
        &self,
        req: Request<csi::v1::ControllerPublishVolumeRequest>,
    ) -> Result<Response<csi::v1::ControllerPublishVolumeResponse>, Status> {
        Err(Status::unimplemented(""))
    }

    async fn controller_unpublish_volume(
        &self,
        req: Request<csi::v1::ControllerUnpublishVolumeRequest>,
    ) -> Result<Response<csi::v1::ControllerUnpublishVolumeResponse>, Status> {
        Err(Status::unimplemented(""))
    }

    async fn validate_volume_capabilities(
        &self,
        req: Request<csi::v1::ValidateVolumeCapabilitiesRequest>,
    ) -> Result<Response<csi::v1::ValidateVolumeCapabilitiesResponse>, Status> {
        Err(Status::unimplemented(""))
    }

    async fn list_volumes(
        &self,
        req: Request<csi::v1::ListVolumesRequest>,
    ) -> Result<Response<csi::v1::ListVolumesResponse>, Status> {
        Err(Status::unimplemented(""))
    }

    async fn get_capacity(
        &self,
        req: Request<csi::v1::GetCapacityRequest>,
    ) -> Result<Response<csi::v1::GetCapacityResponse>, Status> {
        Err(Status::unimplemented(""))
    }

    async fn controller_get_capabilities(
        &self,
        req: Request<csi::v1::ControllerGetCapabilitiesRequest>,
    ) -> Result<Response<csi::v1::ControllerGetCapabilitiesResponse>, Status> {
        Ok(Response::new(csi::v1::ControllerGetCapabilitiesResponse {
            capabilities: vec![csc::rpc::Type::ExpandVolume.into()],
        }))
    }

    async fn create_snapshot(
        &self,
        req: Request<csi::v1::CreateSnapshotRequest>,
    ) -> Result<Response<csi::v1::CreateSnapshotResponse>, Status> {
        Err(Status::unimplemented(""))
    }

    async fn delete_snapshot(
        &self,
        req: Request<csi::v1::DeleteSnapshotRequest>,
    ) -> Result<Response<csi::v1::DeleteSnapshotResponse>, Status> {
        Err(Status::unimplemented(""))
    }

    async fn list_snapshots(
        &self,
        req: Request<csi::v1::ListSnapshotsRequest>,
    ) -> Result<Response<csi::v1::ListSnapshotsResponse>, Status> {
        Err(Status::unimplemented(""))
    }

    async fn get_snapshot(
        &self,
        req: Request<csi::v1::GetSnapshotRequest>,
    ) -> Result<Response<csi::v1::GetSnapshotResponse>, Status> {
        Err(Status::unimplemented(""))
    }

    async fn controller_expand_volume(
        &self,
        req: Request<csi::v1::ControllerExpandVolumeRequest>,
    ) -> Result<Response<csi::v1::ControllerExpandVolumeResponse>, Status> {
        Err(Status::unimplemented(""))
    }

    async fn controller_get_volume(
        &self,
        req: Request<csi::v1::ControllerGetVolumeRequest>,
    ) -> Result<Response<csi::v1::ControllerGetVolumeResponse>, Status> {
        Err(Status::unimplemented(""))
    }

    async fn controller_modify_volume(
        &self,
        req: Request<csi::v1::ControllerModifyVolumeRequest>,
    ) -> Result<Response<csi::v1::ControllerModifyVolumeResponse>, Status> {
        Err(Status::unimplemented(""))
    }
}
