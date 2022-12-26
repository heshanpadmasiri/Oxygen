use storage::{Storage};
use oxygen::{
    oxygen_server::{Oxygen, OxygenServer},
    ClientId, CollectionRequest, CollectionResponse, FileContent, FileRequest, FileResponse,
    RegResponse,
};
use tonic::{Request, Response, Status};
use uuid::Uuid;

mod storage;

pub mod oxygen {
    tonic::include_proto!("oxygen_lib");
}

pub struct OxygenService {
    id: Uuid,
    storage: Storage,
}

impl Default for OxygenService {
    fn default() -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            storage: Storage::new(),
        }
    }
}
#[tonic::async_trait]
impl Oxygen for OxygenService {
    async fn register(&self, request: Request<ClientId>) -> Result<Response<RegResponse>, Status> {
        // TODO: keep track of registered client and state of clients
        let client_id = request.into_inner();
        // TODO: setup logging
        println!("Get register request from: {:?}", &client_id.uuid);

        let reply = RegResponse {
            client_id: client_id.uuid,
            server_id: self.id.to_string(),
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
            collections: self.storage.get_collection_all(),
        }))
    }

    async fn get_collection(
        &self,
        request: Request<CollectionRequest>,
    ) -> Result<Response<CollectionResponse>, Status> {
        match request.into_inner() {
            CollectionRequest {
                client_id: Some(client_id),
                collection_id,
            } => {
                println!(
                    "Get collection request from: {:?} for collection: {:?}",
                    &client_id.uuid, collection_id
                );
                match self.storage.get_collection(collection_id) {
                    Ok(collections) => Ok(Response::new(CollectionResponse {
                        collections: vec![collections],
                    })),
                    Err(()) => Err(Status::new(
                        tonic::Code::InvalidArgument,
                        format!("Failed to find collection with id: {}", collection_id),
                    )),
                }
            }
            CollectionRequest {
                client_id: None,
                collection_id,
            } => {
                let message = format!(
                    "Got collection request for {} without client Id",
                    collection_id
                );
                eprintln!("{}", message);
                Err(Status::new(tonic::Code::InvalidArgument, message))
            }
        }
    }

    async fn get_file(
        &self,
        request: Request<FileRequest>,
    ) -> Result<Response<FileResponse>, Status> {
        match request.into_inner() {
            FileRequest {
                client_id: Some(client_id),
                file_id,
            } => {
                println!(
                    "Get file request from: {:?} for file: {:?}",
                    &client_id.uuid, file_id
                );
                match self.storage.get_file(file_id) {
                    Ok(file) => Ok(Response::new(FileResponse { file: Some(file) })),
                    Err(()) => Err(Status::new(
                        tonic::Code::InvalidArgument,
                        format!("Failed to find file with id: {}", file_id),
                    )),
                }
            }
            FileRequest {
                client_id: None,
                file_id,
            } => {
                let message = format!("Got file request for {} without client Id", file_id);
                eprintln!("{}", message);
                Err(Status::new(tonic::Code::InvalidArgument, message))
            }
        }
    }

    async fn get_file_content(
        &self,
        request: Request<FileRequest>,
    ) -> Result<Response<FileContent>, Status> {
        match request.into_inner() {
            FileRequest {
                client_id: Some(client_id),
                file_id,
            } => {
                println!(
                    "Get file content request from: {:?} for file: {:?}",
                    &client_id.uuid, file_id
                );
                match self.storage.get_file_content(file_id) {
                    Ok(content) => Ok(Response::new(content)),
                    Err(()) => Err(Status::new(
                        tonic::Code::InvalidArgument,
                        format!("Failed to find file with id: {}", file_id),
                    )),
                }
            }
            FileRequest {
                client_id: None,
                file_id,
            } => {
                let message = format!("Got file content request for {} without client Id", file_id);
                eprintln!("{}", message);
                Err(Status::new(tonic::Code::InvalidArgument, message))
            }
        }
    }
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

    use crate::oxygen::{oxygen_client::OxygenClient, ClientId, CollectionRequest, FileRequest};

    #[tokio::test]
    async fn can_initialize_server() {
        let port = 50051;
        let addr = format!("[::1]:{}", port)
            .parse()
            .expect("Hardcoded IP address must be valid");
        let oxygen_service = crate::OxygenService::default();
        let join_handle = tokio::spawn(async move {
            tonic::transport::Server::builder()
                .add_service(crate::oxygen::oxygen_server::OxygenServer::new(
                    oxygen_service,
                ))
                .serve(addr)
                .await
                .expect("failed to start the server");
        });
        join_handle.abort()
    }

    #[tokio::test]
    async fn client_can_register_with_server() {
        let port = 50052;
        let addr = format!("[::1]:{}", port)
            .parse()
            .expect("Hardcoded IP address must be valid");
        let oxygen_service = crate::OxygenService::default();
        let server_id = oxygen_service.id;
        let join_handle = tokio::spawn(async move {
            tonic::transport::Server::builder()
                .add_service(crate::oxygen::oxygen_server::OxygenServer::new(
                    oxygen_service,
                ))
                .serve(addr)
                .await
                .expect("failed to start the server");
        });
        tokio::spawn(async move {
            let mut client = OxygenClient::connect(format!("http://[::1]:{}", port))
                .await
                .expect("failed to create client");
            let uuid = uuid::Uuid::new_v4().to_string();
            let reg_request = tonic::Request::new(crate::oxygen::ClientId {
                uuid: uuid.to_owned(),
            });
            let res = client
                .register(reg_request)
                .await
                .expect("failed to get response from server")
                .into_inner();
            assert_eq!(res.client_id, uuid);
            assert_eq!(res.server_id, server_id.to_string());
        })
        .await
        .expect("failed to run client");
        join_handle.abort()
    }

    #[tokio::test]
    async fn client_can_get_all_collections() {
        let port = 50053;
        let addr = format!("[::1]:{}", port)
            .parse()
            .expect("Hardcoded IP address must be valid");
        let oxygen_service = crate::OxygenService::default();
        let join_handle = tokio::spawn(async move {
            tonic::transport::Server::builder()
                .add_service(crate::oxygen::oxygen_server::OxygenServer::new(
                    oxygen_service,
                ))
                .serve(addr)
                .await
                .expect("failed to start the server");
        });
        tokio::spawn(async move {
            let mut client = OxygenClient::connect(format!("http://[::1]:{}", port))
                .await
                .expect("failed to create client");
            let uuid = uuid::Uuid::new_v4().to_string();
            let _ = client
                .register(tonic::Request::new(crate::oxygen::ClientId {
                    uuid: uuid.to_owned(),
                }))
                .await
                .expect("failed to register with server");
            let collection_res = client
                .get_all_collections(tonic::Request::new(crate::oxygen::ClientId {
                    uuid: uuid.to_owned(),
                }))
                .await
                .expect("failed to get get all collections")
                .into_inner();
            // XXX: hardcoded collection
            assert!(collection_res.collections.len() == 6);
        })
        .await
        .expect("failed to run client");
        join_handle.abort()
    }

    #[tokio::test]
    async fn client_can_get_collection_by_id() {
        let port = 50054;
        let addr = format!("[::1]:{}", port)
            .parse()
            .expect("Hardcoded IP address must be valid");
        let oxygen_service = crate::OxygenService::default();
        let join_handle = tokio::spawn(async move {
            tonic::transport::Server::builder()
                .add_service(crate::oxygen::oxygen_server::OxygenServer::new(
                    oxygen_service,
                ))
                .serve(addr)
                .await
                .expect("failed to start the server");
        });
        tokio::spawn(async move {
            let mut client = OxygenClient::connect(format!("http://[::1]:{}", port))
                .await
                .expect("failed to create client");
            let uuid = uuid::Uuid::new_v4().to_string();
            let _ = client
                .register(tonic::Request::new(crate::oxygen::ClientId {
                    uuid: uuid.to_owned(),
                }))
                .await
                .expect("failed to register with server");
            // XXX: hardcoded storage
            for id in [2, 3, 4, 6, 8, 9] {
                let collection_request = CollectionRequest {
                    client_id: Some(ClientId {
                        uuid: uuid.to_owned(),
                    }),
                    collection_id: id,
                };
                let _ = client
                    .get_collection(tonic::Request::new(collection_request))
                    .await
                    .expect("failed to get get collection");
            }
        })
        .await
        .expect("failed to run client");
        join_handle.abort()
    }

    #[tokio::test]
    async fn server_handle_invalid_collection_ids() {
        let port = 50055;
        let addr = format!("[::1]:{}", port)
            .parse()
            .expect("Hardcoded IP address must be valid");
        let oxygen_service = crate::OxygenService::default();
        let join_handle = tokio::spawn(async move {
            tonic::transport::Server::builder()
                .add_service(crate::oxygen::oxygen_server::OxygenServer::new(
                    oxygen_service,
                ))
                .serve(addr)
                .await
                .expect("failed to start the server");
        });
        tokio::spawn(async move {
            let mut client = OxygenClient::connect(format!("http://[::1]:{}", port))
                .await
                .expect("failed to create client");
            let uuid = uuid::Uuid::new_v4().to_string();
            let _ = client
                .register(tonic::Request::new(crate::oxygen::ClientId {
                    uuid: uuid.to_owned(),
                }))
                .await
                .expect("failed to register with server");
            // XXX: hardcoded storage
            for id in [0, 1, 5, 7, 100, 10000000] {
                let collection_request = CollectionRequest {
                    client_id: Some(ClientId {
                        uuid: uuid.to_owned(),
                    }),
                    collection_id: id,
                };
                let _ = client
                    .get_collection(tonic::Request::new(collection_request))
                    .await
                    .expect_err("server should fail to get invalid collection ids");
            }
        })
        .await
        .expect("failed to run client");
        join_handle.abort()
    }

    #[tokio::test]
    async fn client_can_get_file_by_id() {
        let port = 50056;
        let addr = format!("[::1]:{}", port)
            .parse()
            .expect("Hardcoded IP address must be valid");
        let oxygen_service = crate::OxygenService::default();
        let join_handle = tokio::spawn(async move {
            tonic::transport::Server::builder()
                .add_service(crate::oxygen::oxygen_server::OxygenServer::new(
                    oxygen_service,
                ))
                .serve(addr)
                .await
                .expect("failed to start the server");
        });
        tokio::spawn(async move {
            let mut client = OxygenClient::connect(format!("http://[::1]:{}", port))
                .await
                .expect("failed to create client");
            let uuid = uuid::Uuid::new_v4().to_string();
            let _ = client
                .register(tonic::Request::new(crate::oxygen::ClientId {
                    uuid: uuid.to_owned(),
                }))
                .await
                .expect("failed to register with server");
            // XXX: hardcoded storage
            for id in [0, 1, 5, 7] {
                let file_request = FileRequest {
                    client_id: Some(ClientId {
                        uuid: uuid.to_owned(),
                    }),
                    file_id: id,
                };
                let _ = client
                    .get_file(tonic::Request::new(file_request))
                    .await
                    .expect("failed to get file by id");
            }
        })
        .await
        .expect("failed to run client");
        join_handle.abort()
    }

    #[tokio::test]
    async fn client_can_get_file_content_by_id() {
        let port = 50057;
        let addr = format!("[::1]:{}", port)
            .parse()
            .expect("Hardcoded IP address must be valid");
        let oxygen_service = crate::OxygenService::default();
        let join_handle = tokio::spawn(async move {
            tonic::transport::Server::builder()
                .add_service(crate::oxygen::oxygen_server::OxygenServer::new(
                    oxygen_service,
                ))
                .serve(addr)
                .await
                .expect("failed to start the server");
        });
        tokio::spawn(async move {
            let mut client = OxygenClient::connect(format!("http://[::1]:{}", port))
                .await
                .expect("failed to create client");
            let uuid = uuid::Uuid::new_v4().to_string();
            let _ = client
                .register(tonic::Request::new(crate::oxygen::ClientId {
                    uuid: uuid.to_owned(),
                }))
                .await
                .expect("failed to register with server");
            // XXX: hardcoded storage
            for id in [0, 1, 5, 7] {
                let file_request = FileRequest {
                    client_id: Some(ClientId {
                        uuid: uuid.to_owned(),
                    }),
                    file_id: id,
                };
                let content = client
                    .get_file_content(tonic::Request::new(file_request.clone()))
                    .await
                    .expect("failed to get file content")
                    .into_inner();
                let actual =
                    std::str::from_utf8(&content.body).expect("expect body to be valid utf-8");
                // XXX: hardcoded content
                let file = client
                    .get_file(tonic::Request::new(file_request))
                    .await
                    .expect("failed to get file details")
                    .into_inner()
                    .file
                    .expect("unexpected");
                let expected = format!("# {} content", file.name);
                assert_eq!(actual, expected);
            }
        })
        .await
        .expect("failed to run client");
        join_handle.abort()
    }

    #[tokio::test]
    async fn server_handle_invalid_file_ids() {
        let port = 50058;
        let addr = format!("[::1]:{}", port)
            .parse()
            .expect("Hardcoded IP address must be valid");
        let oxygen_service = crate::OxygenService::default();
        let join_handle = tokio::spawn(async move {
            tonic::transport::Server::builder()
                .add_service(crate::oxygen::oxygen_server::OxygenServer::new(
                    oxygen_service,
                ))
                .serve(addr)
                .await
                .expect("failed to start the server");
        });
        tokio::spawn(async move {
            let mut client = OxygenClient::connect(format!("http://[::1]:{}", port))
                .await
                .expect("failed to create client");
            let uuid = uuid::Uuid::new_v4().to_string();
            let _ = client
                .register(tonic::Request::new(crate::oxygen::ClientId {
                    uuid: uuid.to_owned(),
                }))
                .await
                .expect("failed to register with server");
            // XXX: hardcoded storage
            for id in [2, 3, 4, 6, 8, 9, 100, 10000000] {
                let file_request = FileRequest {
                    client_id: Some(ClientId {
                        uuid: uuid.to_owned(),
                    }),
                    file_id: id,
                };
                let _ = client
                    .get_file(tonic::Request::new(file_request))
                    .await
                    .expect_err("server should fail to get invalid file ids");
            }
        })
        .await
        .expect("failed to run client");
        join_handle.abort()
    }
}
