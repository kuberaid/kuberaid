use kuberaid_api::v1::{
    GetPoolRequest, ListPoolsRequest, WatchRequest, manager_client::ManagerClient,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = ManagerClient::connect("http://[::1]:50051").await?;

    let resp = client.list_pools(ListPoolsRequest {}).await?.into_inner();
    println!("{resp:?}");

    let resp = client
        .get_pool(GetPoolRequest {
            name: "lab".to_string(),
        })
        .await?
        .into_inner();
    println!("{resp:?}");

    let mut events = client.watch_state(WatchRequest {}).await?.into_inner();
    while let Ok(Some(event)) = events.message().await {
        println!("event: {event:?}");
    }

    Ok(())
}
