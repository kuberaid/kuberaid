use std::collections::HashMap;

use k8s_csi::{
    self as csi,
    v1::{identity_server::Identity, plugin_capability as pc},
};
use tonic::{Request, Response, Status};

use super::CSIPlugin;
use crate::{DRIVER_NAME, DRIVER_VERSION};

#[tonic::async_trait]
impl Identity for CSIPlugin {
    async fn get_plugin_info(
        &self,
        _req: Request<csi::v1::GetPluginInfoRequest>,
    ) -> Result<Response<csi::v1::GetPluginInfoResponse>, Status> {
        Ok(Response::new(csi::v1::GetPluginInfoResponse {
            name: DRIVER_NAME.to_string(),
            vendor_version: DRIVER_VERSION.to_string(),
            manifest: HashMap::new(),
        }))
    }

    async fn get_plugin_capabilities(
        &self,
        _req: Request<csi::v1::GetPluginCapabilitiesRequest>,
    ) -> Result<Response<csi::v1::GetPluginCapabilitiesResponse>, Status> {
        Ok(Response::new(csi::v1::GetPluginCapabilitiesResponse {
            capabilities: vec![
                pc::service::Type::ControllerService.into(),
                pc::service::Type::VolumeAccessibilityConstraints.into(),
                pc::volume_expansion::Type::Online.into(),
            ],
        }))
    }

    async fn probe(
        &self,
        _req: Request<csi::v1::ProbeRequest>,
    ) -> Result<Response<csi::v1::ProbeResponse>, Status> {
        // let (healthy, ready) = *self
        //     .0
        //     .healthy
        //     .read()
        //     .map_err(|e| Status::internal(format!("{e}")))?;
        if true {
            Ok(Response::new(csi::v1::ProbeResponse { ready: None }))
        } else {
            Err(Status::failed_precondition("Failed health check."))
        }
    }
}
