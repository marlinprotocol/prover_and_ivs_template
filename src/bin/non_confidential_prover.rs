use anyhow::{anyhow, Result};
use async_trait::async_trait;
use dotenv::dotenv;
use kalypso_generator::generator::{start_non_confidential_proving_server, GeneratorTrait};
use kalypso_generator::models::*;
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
    env_var!(ivs_url, "IVS_URL");
    env_var!(prover_url, "PROVER_URL");
    env_var!(polling_interval, "POLLING_INTERVAL");

    let http_rpc_url = env::var("HTTP_RPC_URL")
        .or_else(|_| env::var("RPC_URL"))
        .expect("HTTP_RPC_URL or RPC_URL is not set");

    // Prepare a vector to hold the task handles.
    let mut handles = vec![];

    // Clone prover_url as it will be moved into the first task.
    let prover_url_clone = prover_url.clone();
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

        // Create a listener with the given configuration.
        let listener =
            kalypso_listener::job_creator::JobCreator::simple_listener_for_non_confidential_prover(
                generator,
                market_id.into(),
                http_rpc_url.into(),
                gas_key,
                proof_market_place.into(),
                generator_registry.into(),
                start_block,
                chain_id,
                prover_url_clone,
                ivs_url,
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

    let null_prover = NullProver::default();
    start_non_confidential_proving_server(prover_url.as_ref(), null_prover).await?;

    for handle in handles {
        handle
            .await
            .map_err(|e| anyhow!("Task join error: {:?}", e))??;
    }

    println!("All tasks completed or shutdown.");
    Ok(())
}

// Dummy implementation of the NullProver.
#[derive(Default)]
struct NullProver;

#[async_trait]
impl GeneratorTrait for NullProver {
    async fn generate_proof(&self, _input: InputPayload) -> GenerateProofResponse {
        // Actual proof generation logic here.
        unimplemented!()
    }

    async fn benchmark(&self) -> BenchmarkResponse {
        // Actual benchmarking logic here.
        unimplemented!()
    }
}
