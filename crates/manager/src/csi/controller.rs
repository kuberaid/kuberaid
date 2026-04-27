use k8s_csi::v1::{
    ControllerExpandVolumeRequest, ControllerExpandVolumeResponse,
    ControllerGetCapabilitiesRequest, ControllerGetCapabilitiesResponse,
    ControllerGetVolumeRequest, ControllerGetVolumeResponse, ControllerModifyVolumeRequest,
    ControllerModifyVolumeResponse, ControllerPublishVolumeRequest,
    ControllerPublishVolumeResponse, ControllerUnpublishVolumeRequest,
    ControllerUnpublishVolumeResponse, CreateSnapshotRequest, CreateSnapshotResponse,
    CreateVolumeRequest, CreateVolumeResponse, DeleteSnapshotRequest, DeleteSnapshotResponse,
    DeleteVolumeRequest, DeleteVolumeResponse, GetCapacityRequest, GetCapacityResponse,
    GetSnapshotRequest, GetSnapshotResponse, ListSnapshotsRequest, ListSnapshotsResponse,
    ListVolumesRequest, ListVolumesResponse, ValidateVolumeCapabilitiesRequest,
    ValidateVolumeCapabilitiesResponse, controller_server::Controller,
};

use crate::csi::CSIPlugin;

use tonic::*;

#[tonic::async_trait]
impl Controller for CSIPlugin {
    async fn create_volume(
        &self,
        req: Request<CreateVolumeRequest>,
    ) -> Result<Response<CreateVolumeResponse>, Status> {
        Err(Status::unimplemented(""))
    }

    async fn delete_volume(
        &self,
        req: Request<DeleteVolumeRequest>,
    ) -> Result<Response<DeleteVolumeResponse>, Status> {
        Err(Status::unimplemented(""))
    }

    async fn controller_publish_volume(
        &self,
        req: Request<ControllerPublishVolumeRequest>,
    ) -> Result<Response<ControllerPublishVolumeResponse>, Status> {
        Err(Status::unimplemented(""))
    }

    async fn controller_unpublish_volume(
        &self,
        req: Request<ControllerUnpublishVolumeRequest>,
    ) -> Result<Response<ControllerUnpublishVolumeResponse>, Status> {
        Err(Status::unimplemented(""))
    }

    async fn validate_volume_capabilities(
        &self,
        req: Request<ValidateVolumeCapabilitiesRequest>,
    ) -> Result<Response<ValidateVolumeCapabilitiesResponse>, Status> {
        Err(Status::unimplemented(""))
    }

    async fn list_volumes(
        &self,
        req: Request<ListVolumesRequest>,
    ) -> Result<Response<ListVolumesResponse>, Status> {
        Err(Status::unimplemented(""))
    }

    async fn get_capacity(
        &self,
        req: Request<GetCapacityRequest>,
    ) -> Result<Response<GetCapacityResponse>, Status> {
        Err(Status::unimplemented(""))
    }

    async fn controller_get_capabilities(
        &self,
        req: Request<ControllerGetCapabilitiesRequest>,
    ) -> Result<Response<ControllerGetCapabilitiesResponse>, Status> {
        Ok(Response::new(ControllerGetCapabilitiesResponse {
            capabilities: vec![],
        }))
    }

    async fn create_snapshot(
        &self,
        req: Request<CreateSnapshotRequest>,
    ) -> Result<Response<CreateSnapshotResponse>, Status> {
        Err(Status::unimplemented(""))
    }

    async fn delete_snapshot(
        &self,
        req: Request<DeleteSnapshotRequest>,
    ) -> Result<Response<DeleteSnapshotResponse>, Status> {
        Err(Status::unimplemented(""))
    }

    async fn list_snapshots(
        &self,
        req: Request<ListSnapshotsRequest>,
    ) -> Result<Response<ListSnapshotsResponse>, Status> {
        Err(Status::unimplemented(""))
    }

    async fn get_snapshot(
        &self,
        req: Request<GetSnapshotRequest>,
    ) -> Result<Response<GetSnapshotResponse>, Status> {
        Err(Status::unimplemented(""))
    }

    async fn controller_expand_volume(
        &self,
        req: Request<ControllerExpandVolumeRequest>,
    ) -> Result<Response<ControllerExpandVolumeResponse>, Status> {
        Err(Status::unimplemented(""))
    }

    async fn controller_get_volume(
        &self,
        req: Request<ControllerGetVolumeRequest>,
    ) -> Result<Response<ControllerGetVolumeResponse>, Status> {
        Err(Status::unimplemented(""))
    }

    async fn controller_modify_volume(
        &self,
        req: Request<ControllerModifyVolumeRequest>,
    ) -> Result<Response<ControllerModifyVolumeResponse>, Status> {
        Err(Status::unimplemented(""))
    }
}
