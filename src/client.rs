use oxygen::{oxygen_client::OxygenClient, ClientId};

pub mod oxygen {
    tonic::include_proto!("oxygen_lib");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let uuid = uuid::Uuid::new_v4().to_string();
    let mut client = OxygenClient::connect("http://[::1]:50050").await?;
    let reg_request = tonic::Request::new(ClientId { uuid });
    let reg_response = client.register(reg_request).await?;
    println!("REG RES = {:?}", reg_response);
    Ok(())
}
