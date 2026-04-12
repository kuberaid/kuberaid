use kuberaid_common::grpc::{ListPoolsRequest, agent_client::AgentClient};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = AgentClient::connect("http://[::1]:50051").await?;

    let resp = client.list_pools(ListPoolsRequest {}).await?.into_inner();

    println!("{resp:?}");
    Ok(())
}
