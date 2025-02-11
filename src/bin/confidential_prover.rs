use anyhow::{anyhow, Result};
use async_trait::async_trait;
use dotenv::dotenv;
use kalypso_generator::generator::GeneratorTrait;
use kalypso_generator::models::*;
use kalypso_ivs::ivs::{start_confidential_proving_server, IVSTrait};
use kalypso_ivs::models::*;
use tokio::fs;

use std::env;

macro_rules! env_var {
    ($var:ident, $key:expr) => {
        let $var = env::var($key).expect(&format!("{} is not set", $key));
    };
}

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables and initialize the logger.
    dotenv().ok();
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    // Get environment variables.
    env_var!(generator, "GENERATOR_ADDRESS");
    env_var!(gas_key, "GAS_KEY");
    env_var!(market_id, "MARKET_ID");
    env_var!(proof_market_place, "PROOF_MARKETPLACE_ADDRESS");
    env_var!(generator_registry, "GENERATOR_REGISTRY_ADDRESS");
    env_var!(start_block, "START_BLOCK");
    env_var!(chain_id, "CHAIN_ID");
    env_var!(max_parallel_proofs, "MAX_PARALLEL_PROOFS");
    env_var!(prover_port, "PROVER_PORT");
    env_var!(polling_interval, "POLLING_INTERVAL");

    let http_rpc_url = env::var("HTTP_RPC_URL")
        .or_else(|_| env::var("RPC_URL"))
        .expect("HTTP_RPC_URL or RPC_URL is not set");

    // Prepare a vector to hold the task handles.
    let mut handles = vec![];

    // Clone prover_url as it will be moved into the first task.
    let prover_port_clone = prover_port.clone();

    let ecies_private_key = match fs::read("/app/secp.sec").await {
        Ok(key) => key,
        Err(_) => fs::read("./app/secp.sec").await?,
    };

    let ecies_private_key_clone = ecies_private_key.clone();

    let handle_1 = tokio::spawn(async move {
        // Parse some environment variables.
        let start_block: u64 = start_block
            .parse()
            .expect("Cannot parse start_block as u64");
        let chain_id: u64 = chain_id.parse().expect("Cannot parse chain_id as u64");
        let max_parallel_proofs: usize = max_parallel_proofs.parse().unwrap_or(1);

        log::info!(
            "Start Block: {}, Max Parallel Requests: {}",
            start_block,
            max_parallel_proofs
        );

        // Parse polling_interval using the ? operator so that any error is propagated.
        let polling_interval_val: u64 = polling_interval.parse()?;

        let listener =
            kalypso_listener::job_creator::JobCreator::simple_listener_for_confidential_prover(
                generator,
                hex::encode(ecies_private_key_clone),
                market_id,
                http_rpc_url,
                gas_key,
                proof_market_place,
                generator_registry,
                start_block,
                chain_id,
                prover_port_clone,
                false,
                max_parallel_proofs,
                false,
                9999,
                polling_interval_val,
            );

        // Run the listener.
        listener.run().await
    });
    handles.push(handle_1);

    let null_confidential_prover = NullConfProver::default();
    start_confidential_proving_server(
        format!("localhost:{}", prover_port.to_string()).as_ref(),
        null_confidential_prover,
        ecies_private_key,
    )
    .await?;

    for handle in handles {
        handle
            .await
            .map_err(|e| anyhow!("Task join error: {:?}", e))??;
    }

    println!("All tasks completed or shutdown.");
    Ok(())
}

#[derive(Default)]
struct NullConfProver;

#[async_trait]
impl GeneratorTrait for NullConfProver {
    async fn generate_proof(&self, _input: InputPayload) -> GenerateProofResponse {
        // Actual logic here
        unimplemented!()
    }

    async fn benchmark(&self) -> BenchmarkResponse {
        // Actual logic here
        unimplemented!()
    }
}

#[async_trait]
impl IVSTrait for NullConfProver {
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
