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

    env_var!(port, "GENERATOR_CLIENT_PORT"); //5000 comes from env
    env_var!(_supervisord_path, "SUPERVISORD_PATH"); // ./app/supervisord comes from env

    let port = port.parse().unwrap();

    let enclave_key = match fs::read("/app/ecdsa.sec").await {
        Ok(key) => key,
        Err(_) => match fs::read("./app/ecdsa.sec").await {
            Ok(key) => key,
            Err(_) => {
                panic!("ecdsa.sec not found.");
            }
        },
    };

    let server = client::GeneratorClient::new(hex::encode(enclave_key), port);

    server.start(false).await.unwrap();

    Ok(())
}

// {
//     "generator_config": [
//       {
//         "address": "0xa1b2c3d4e5f60718293a4b5c6d7e8f9012345678",
//         "data": "sample payload data",
//         "supported_markets": ["1"]
//       }
//     ],
//     "runtime_config": {
//       "ws_url": "wss://arbitrum-sepolia-rpc.publicnode.com",
//       "http_url": "https://sepolia-rollup.arbitrum.io/rpc",
//       "private_key": "0x1111111111111111111111111111111111111111111111111111111111111111",
//       "proof_market_place": "0xC05d689B341d84900f0d0CE36f35aDAbfB57F68d",
//       "generator_registry": "0x4743a2c7a96C9FBED8b7eAD980aD01822F9711Db",
//       "start_block": 100,
//       "chain_id": 1,
//       "payment_token": "0x8230d71d809718132C2054704F5E3aF1b86B669C",
//       "staking_token": "0xB5570D4D39dD20F61dEf7C0d6846790360b89a18",
//       "attestation_verifier": "0xB5570D4D39dD20F61dEf7C0d6846790360b89a18",
//       "entity_registry": "0x457D42573096b339bA48Be576e9Db4Fc5F186091",
//       "markets": {
//         "1": {
//           "port": "8080",
//           "ivs_url": "http://ivs.market1.com"
//         }
//       }
//     }
//   }
