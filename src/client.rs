use oxygen::{oxygen_client::OxygenClient, ClientId};

pub mod oxygen {
    tonic::include_proto!("oxygen_lib");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let uuid = uuid::Uuid::new_v4().to_string();
    let mut client = OxygenClient::connect("http://[::1]:50051").await?;

    let request = tonic::Request::new(Req {
        msg: "test message".to_owned(),
        id: 0,
    });

    let response = client.hello(request).await?;
    println!("RESPONSE={:?}", response);
    Ok(())
}
