use anyhow::Result;
use async_trait::async_trait;
use dotenv::dotenv;
use kalypso_generator::models::InputPayload;
use kalypso_ivs::ivs::{start_ivs_server, IVSTrait};
use kalypso_ivs::models::*;
use tokio::fs;

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables and initialize the logger.
    dotenv().ok();
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let ecies_private_key = match fs::read("/app/secp.sec").await {
        Ok(key) => key,
        Err(_) => fs::read("./app/secp.sec").await?,
    };

    let null_confidential_prover = IVS::default();
    start_ivs_server("0.0.0.0:3030", null_confidential_prover, ecies_private_key).await?;

    Ok(())
}

#[derive(Default)]
struct IVS;

#[async_trait]
impl IVSTrait for IVS {
    async fn check_inputs(&self, _input: InputPayload) -> CheckInputResponse {
        // Actual logic here
        unimplemented!()
    }
    async fn check_inputs_and_proof(
        &self,
        _input: VerifyInputsAndProof,
    ) -> VerifyInputAndProofResponse {
        // Actual Logic Here
        unimplemented!()
    }
}
