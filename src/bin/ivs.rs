use anyhow::Result;
use async_trait::async_trait;
use dotenv::dotenv;
use kalypso_generator::models::InputPayload;
use kalypso_ivs::ivs::{start_ivs_server, IVSTrait};
use kalypso_ivs::models::*;
use std::env;
use tokio::fs;

macro_rules! env_var {
    ($var:ident, $key:expr) => {
        let $var = env::var($key).expect(&format!("{} is not set", $key));
    };
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    env_var!(ivs_port, "IVS_PORT");
    let ecies_private_key = match fs::read("/app/secp.sec").await {
        Ok(key) => key,
        Err(_) => fs::read("./app/secp.sec").await?,
    };

    let null_confidential_prover = IVS::default();
    start_ivs_server(
        format!("0.0.0.0:{}", ivs_port).as_ref(),
        null_confidential_prover,
        ecies_private_key,
    )
    .await?;

    Ok(())
}

#[derive(Default)]
struct IVS;

#[async_trait]
impl IVSTrait for IVS {
    async fn check_inputs(&self, _input: InputPayload) -> CheckInputResponse {
        unimplemented!()
    }
    async fn check_inputs_and_proof(
        &self,
        _input: VerifyInputsAndProof,
    ) -> VerifyInputAndProofResponse {
        unimplemented!()
    }
}
