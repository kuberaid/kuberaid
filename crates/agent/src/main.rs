#![warn(clippy::pedantic)]

use tonic::{Request, Response, Status, transport::Server};

use kuberaid_common::grpc::{
    HelloReply, HelloRequest,
    greeter_server::{Greeter, GreeterServer},
};

#[derive(Debug, Default)]
pub struct MyGreeter {}

#[tonic::async_trait]
impl Greeter for MyGreeter {
    async fn say_hello(
        &self,
        request: Request<HelloRequest>, // Accept request of type HelloRequest
    ) -> Result<Response<HelloReply>, Status> {
        // Return an instance of type HelloReply
        println!("Got a request: {:?}", request);

        let reply = HelloReply {
            message: format!("Hello {}!", request.into_inner().name), // We must use .into_inner() as the fields of gRPC requests and responses are private
        };

        Ok(Response::new(reply)) // Send back our formatted greeting
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    let agent = MyGreeter::default();

    Server::builder()
        .add_service(GreeterServer::new(agent))
        .serve(addr)
        .await?;

    Ok(())
}
