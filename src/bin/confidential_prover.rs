use anyhow::{anyhow, Result};
use async_trait::async_trait;
use dotenv::dotenv;
use kalypso_generator::generator::GeneratorTrait;
use kalypso_generator::models::*;
use kalypso_generator_client::model::{GeneratorConfigFile, RuntimeConfigFile};
use kalypso_ivs::ivs::{start_confidential_proving_server, IVSTrait};
use kalypso_ivs::models::*;
use std::{env, fs};

macro_rules! env_var {
    ($var:ident, $key:expr) => {
        let $var = env::var($key).expect(&format!("{} is not set", $key));
    };
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    env_var!(max_parallel_proofs, "MAX_PARALLEL_PROOFS"); // 1 from env for now
    env_var!(polling_interval, "POLLING_INTERVAL"); // 10000 from env for now
    env_var!(prometheus_port, "PROMETHEUS_PORT"); // 9999 from env for now

    let mut handles = vec![];

    let generator_config_path = "/app/generator_config/generator_config.json";
    let runtime_config_path = "/app/generator_config/runtime_config.json";

    // Poll until both config files exist
    loop {
        let generator_exists = std::path::Path::new(generator_config_path).exists();
        let runtime_exists = std::path::Path::new(runtime_config_path).exists();

        if generator_exists && runtime_exists {
            println!("Generator and runtime config files found");
            break;
        }

        println!("Waiting for generator and runtime config files to be created...");

        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    }

    let handle_1 = tokio::spawn(async move {
        let max_parallel_proofs: usize = max_parallel_proofs.parse().unwrap_or(1);

        let polling_interval_val: u64 = polling_interval.parse()?;
        let prometheus_port = prometheus_port.parse()?;

        let listener = kalypso_listener::job_creator::JobCreator::from_config_paths(
            generator_config_path,
            runtime_config_path,
            false,
            max_parallel_proofs,
            false,
            prometheus_port,
            polling_interval_val,
        )
        .unwrap();

        listener.run().await
    });
    handles.push(handle_1);

    let null_confidential_prover = NullConfProver::default();

    let runtime_config_content = fs::read_to_string(runtime_config_path)?;
    let runtime_config_file: RuntimeConfigFile = serde_json::from_str(&runtime_config_content)?;

    let market_details = runtime_config_file
        .runtime_config
        .markets
        .values()
        .next()
        .ok_or("No market details found in runtime config")
        .unwrap();
    let prover_port = market_details.port.clone();

    let generator_config_content = fs::read_to_string(generator_config_path)?;
    let generator_config_file: GeneratorConfigFile =
        serde_json::from_str(&generator_config_content)?;

    // Extract the first generator config.
    let generator_config = generator_config_file
        .generator_config
        .get(0)
        .ok_or("No generator config found")
        .unwrap();

    // Process ecies_private_key: remove a leading "0x" if present and decode from hex.
    let mut ecies_key_str = generator_config.ecies_private_key.clone();
    if ecies_key_str.starts_with("0x") || ecies_key_str.starts_with("0X") {
        ecies_key_str = ecies_key_str
            .trim_start_matches("0x")
            .trim_start_matches("0X")
            .to_string();
    }
    let ecies_private_key = hex::decode(ecies_key_str)?;

    start_confidential_proving_server(
        format!("0.0.0.0:{}", prover_port.to_string()).as_ref(),
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
        unimplemented!()
    }

    async fn benchmark(&self) -> BenchmarkResponse {
        unimplemented!()
    }
}

#[async_trait]
impl IVSTrait for NullConfProver {
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
