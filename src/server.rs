use oxygen::{
    oxygen_server::{Oxygen, OxygenServer},
    ClientId, Collection, CollectionRequest, CollectionResponse, RegResponse,
};
use tonic::{Request, Response, Status};
use uuid::Uuid;

pub mod oxygen {
    tonic::include_proto!("oxygen_lib");
}

pub struct OxygenService {
    id: Uuid,
}

impl Default for OxygenService {
    fn default() -> Self {
        Self { id: uuid::Uuid::new_v4() }
    }
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
                match get_collection(collection_id) {
                    Ok(collections) => {
                        Ok(Response::new(CollectionResponse {
                            successful: true,
                            collections
                        }))
                    },
                    Err(()) => {
                        Err(Status::new(tonic::Code::InvalidArgument, format!("Failed to find collection with id: {}", collection_id)))
                    }
                }
            }
            CollectionRequest { client_id: None, collection_id } => {
                let message = format!("Got collection request for {} without client Id", collection_id);
                eprintln!("{}", message);
                Err(Status::new(tonic::Code::InvalidArgument, message))
            }
        }
    }
}

// FIXME:
fn get_collection_all() -> Vec<Collection> {
    vec![]
}

fn get_collection(_id: u64) -> Result<Vec<Collection>, ()> {
    Ok(vec![])
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    let oxygen_service = OxygenService::default();
    tonic::transport::Server::builder()
        .add_service(OxygenServer::new(oxygen_service))
        .serve(addr)
        .await?;
    Ok(())
}
