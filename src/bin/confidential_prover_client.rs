use dotenv::dotenv;
use kalypso_generator_client::client;
use tokio::fs;

use std::env;

macro_rules! env_var {
    ($var:ident, $key:expr) => {
        let $var = env::var($key).expect(&format!("{} is not set", $key));
    };
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    env_var!(port, "GENERATOR_CLIENT_PORT");

    let port = port.parse().unwrap();

    let enclave_key = match fs::read("/app/secp.sec").await {
        Ok(key) => key,
        Err(_) => fs::read("./app/secp.sec").await?,
    };

    let server = client::GeneratorClient::new(hex::encode(enclave_key), port);

    server.start(false).await.unwrap();

    Ok(())
}
