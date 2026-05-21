use std::collections::HashMap;

use k8s_csi::v1::{
    GetPluginCapabilitiesRequest, GetPluginCapabilitiesResponse, GetPluginInfoRequest,
    GetPluginInfoResponse, ProbeRequest, ProbeResponse, identity_server::Identity,
    plugin_capability as pc,
};
use tonic::{Request, Response, Status};

use crate::{DRIVER_NAME, DRIVER_VERSION};

use super::CSIPlugin;

#[tonic::async_trait]
impl Identity for CSIPlugin {
    async fn get_plugin_info(
        &self,
        _req: Request<GetPluginInfoRequest>,
    ) -> Result<Response<GetPluginInfoResponse>, Status> {
        Ok(Response::new(GetPluginInfoResponse {
            name: DRIVER_NAME.to_string(),
            vendor_version: DRIVER_VERSION.to_string(),
            manifest: HashMap::new(),
        }))
    }

    async fn get_plugin_capabilities(
        &self,
        _req: Request<GetPluginCapabilitiesRequest>,
    ) -> Result<Response<GetPluginCapabilitiesResponse>, Status> {
        Ok(Response::new(GetPluginCapabilitiesResponse {
            capabilities: vec![
                pc::service::Type::ControllerService.into(),
                pc::service::Type::VolumeAccessibilityConstraints.into(),
                pc::volume_expansion::Type::Online.into(),
            ],
        }))
    }

    async fn probe(&self, _req: Request<ProbeRequest>) -> Result<Response<ProbeResponse>, Status> {
        // let (healthy, ready) = *self
        //     .0
        //     .healthy
        //     .read()
        //     .map_err(|e| Status::internal(format!("{e}")))?;
        if true {
            Ok(Response::new(ProbeResponse { ready: None }))
        } else {
            Err(Status::failed_precondition("Failed health check."))
        }
    }
}
