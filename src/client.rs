use oxygen::{oxygen_client::OxygenClient, Req};

pub mod oxygen {
    tonic::include_proto!("oxygen");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = OxygenClient::connect("http://[::1]:50051").await?;

    let request = tonic::Request::new(Req {
        msg: "test message".to_owned(),
        id: 0,
    });

    let response = client.hello(request).await?;
    println!("RESPONSE={:?}", response);
    Ok(())
}
