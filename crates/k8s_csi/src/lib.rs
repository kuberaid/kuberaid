pub mod v1 {
    tonic::include_proto!("csi.v1");

    impl From<node_service_capability::rpc::Type> for NodeServiceCapability {
        fn from(value: node_service_capability::rpc::Type) -> Self {
            NodeServiceCapability {
                r#type: Some(node_service_capability::Type::Rpc(
                    node_service_capability::Rpc {
                        r#type: value as i32,
                    },
                )),
            }
        }
    }

    impl From<plugin_capability::Type> for PluginCapability {
        fn from(value: plugin_capability::Type) -> Self {
            Self {
                r#type: Some(value),
            }
        }
    }

    impl From<plugin_capability::service::Type> for plugin_capability::Service {
        fn from(value: plugin_capability::service::Type) -> Self {
            Self {
                r#type: value as i32,
            }
        }
    }

    impl From<plugin_capability::service::Type> for PluginCapability {
        fn from(value: plugin_capability::service::Type) -> Self {
            plugin_capability::Type::Service(value.into()).into()
        }
    }
}
