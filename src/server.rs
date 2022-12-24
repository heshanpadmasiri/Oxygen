use oxygen::{
    oxygen_server::{Oxygen, OxygenServer},
    ClientId, Collection, CollectionRequest, CollectionResponse, RegResponse,
};
use tonic::{Request, Response, Status};
use uuid::Uuid;

pub mod oxygen {
    tonic::include_proto!("oxygen_lib");
}

#[derive(Default)]
pub struct OxygenService {
    id: Uuid,
}
#[tonic::async_trait]
impl Oxygen for OxygenService {
    async fn register(&self, request: Request<ClientId>) -> Result<Response<RegResponse>, Status> {
        // TODO: keep track of registered client and state of clients
        let client_id = request.into_inner();
        // TODO: setup logging
        println!("Get register request from: {:?}", &client_id.uuid);

        let reply = Res {
            msg: format!("Recieved : {}", req.msg),
            successful: true,
        };
        Ok(Response::new(reply))
    }

    async fn get_all_collections(
        &self,
        request: Request<ClientId>,
    ) -> Result<Response<CollectionResponse>, Status> {
        // TODO: keep track of registered client and state of clients
        let client_id = request.into_inner();
        // TODO: setup logging
        println!("Get all collection request from: {:?}", &client_id.uuid);

        Ok(Response::new(CollectionResponse {
            successful: true,
            collections: get_collection_all(),
        }))
    }

    async fn get_collection(
        &self,
        request: Request<CollectionRequest>,
    ) -> Result<Response<CollectionResponse>, Status> {
        match request.into_inner() {
            CollectionRequest {
                client_id: Some(client),
                collection_id,
            } => {
                println!(
                    "Get collection request from: {:?} for collection: {:?}",
                    &client.uuid, collection_id
                );
                Ok(Response::new(CollectionResponse {
                    successful: true,
                    collections: get_collection(collection_id),
                }))
            }
            _ => {
                let message = format!("Unexpected request for get collection").to_owned();
                eprintln!("{message}");
                Err(Status::new(tonic::Code::InvalidArgument, message))
            }
        }
    }
}

// FIXME:
fn get_collection_all() -> Vec<Collection> {
    vec![]
}

fn get_collection(_id: u64) -> Vec<Collection> {
    vec![]
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // TODO: take this from a config file
    let addr = "[::1]:50051".parse()?;
    let oxygen_service = OxygenService::default();
    tonic::transport::Server::builder()
        .add_service(OxygenServer::new(oxygen_service))
        .serve(addr)
        .await?;

    Ok(())
}
