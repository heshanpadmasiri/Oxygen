use oxygen::{
    oxygen_server::{Oxygen, OxygenServer},
    Req, Res,
};
use tonic::{transport::Server, Request, Response, Status};

pub mod oxygen {
    tonic::include_proto!("oxygen");
}

#[derive(Default)]
pub struct OxygenService {}

#[tonic::async_trait]
impl Oxygen for OxygenService {
    async fn hello(&self, request: Request<Req>) -> Result<Response<Res>, Status> {
        println!("Got request: {:?}", request);
        let req = request.into_inner();

        let reply = Res {
            msg: format!("Recieved : {}", req.msg),
            successful: true,
        };
        Ok(Response::new(reply))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    let oxygen_service = OxygenService::default();
    Server::builder()
        .add_service(OxygenServer::new(oxygen_service))
        .serve(addr)
        .await?;

    Ok(())
}
