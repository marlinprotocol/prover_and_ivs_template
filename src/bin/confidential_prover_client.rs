use dotenv::dotenv;
use kalypso_generator_client::client;
use tokio::fs;

use std::env;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let port: u16 = env::var("GENERATOR_CLIENT_PORT")
        .unwrap_or_else(|_| "1500".to_string())
        .parse::<u16>()
        .expect("GENERATOR_CLIENT_PORT must be a valid number");

    let enclave_key = match fs::read("/app/secp.sec").await {
        Ok(key) => key,
        Err(_) => fs::read("./app/secp.sec").await?,
    };

    let server = client::GeneratorClient::new(hex::encode(enclave_key), port);

    server.start(false).await.unwrap();

    Ok(())
}