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

// TODO: factor these functions to separate module
// FIXME:
fn get_collection_all() -> Vec<Collection> {
    vec![]
}

fn get_collection(_id: u64) -> Result<Vec<Collection>, ()> {
    Ok(vec![])
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50050".parse()?;
    let oxygen_service = OxygenService::default();
    tonic::transport::Server::builder()
        .add_service(OxygenServer::new(oxygen_service))
        .serve(addr)
        .await?;
    Ok(())
}

#[cfg(test)]
mod tests {

    use crate::oxygen::oxygen_client::OxygenClient;

    #[tokio::test]
    async fn can_initialize_server() {
        let port = 50051;
        let addr = format!("[::1]:{}", port).parse().expect("Hardcoded IP address must be valid");
        let oxygen_service = crate::OxygenService::default();
        let join_handle = tokio::spawn(async move {
            tonic::transport::Server::builder()
                .add_service(crate::oxygen::oxygen_server::OxygenServer::new(oxygen_service))
                .serve(addr)
                .await
                .expect("failed to start the server");
        });
        join_handle.abort()
    }

    #[tokio::test]
    async fn client_can_register_with_server() {
        let port = 50052;
        let addr = format!("[::1]:{}", port).parse().expect("Hardcoded IP address must be valid");
        let oxygen_service = crate::OxygenService::default();
        let server_id = oxygen_service.id;
        let join_handle = tokio::spawn(async move {
            tonic::transport::Server::builder()
                .add_service(crate::oxygen::oxygen_server::OxygenServer::new(oxygen_service))
                .serve(addr)
                .await
                .expect("failed to start the server");
        });
        tokio::spawn(async move {
            let mut client = OxygenClient::connect(format!("http://[::1]:{}", port)).await.expect("failed to create client");
            let uuid = uuid::Uuid::new_v4().to_string();
            let reg_request = tonic::Request::new(crate::oxygen::ClientId { uuid: uuid.to_owned() });
            let res = client.register(reg_request).await.expect("failed to get response from server").into_inner();
            assert_eq!(res.client_id, uuid);
            assert_eq!(res.server_id, server_id.to_string());
            assert!(res.successful)
        }).await.expect("failed to run client");
        join_handle.abort()
    }
}