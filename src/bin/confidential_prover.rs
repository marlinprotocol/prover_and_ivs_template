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

    let generator_exists = std::path::Path::new(generator_config_path).exists();
    let runtime_exists = std::path::Path::new(runtime_config_path).exists();

    if !generator_exists || !runtime_exists {
        panic!("Generator or runtime config files not found");
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

    let null_confidential_prover = NoirProver::new();

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
struct NoirProver {
    toml_path: String,
    output_path: String,
    lock: tokio::sync::Mutex<()>,
}

impl NoirProver {
    fn new() -> Self {
        Self {
            toml_path: "/app/noir_enclave_setup/hello_world".to_string(),
            output_path: "/app/noir_enclave_setup/hello_world/target/hello_world.proof"
                .to_string(),
            lock: tokio::sync::Mutex::new(()),
        }
    }
}

#[async_trait]
impl GeneratorTrait for NoirProver {
    async fn generate_proof(&self, inputs: InputPayload) -> GenerateProofResponse {
        let _lock = self.lock.lock().await;
        write_private_inputs_to_toml(inputs.clone(), &self.toml_path).unwrap();
        let proof_path = execute_prove_command(inputs.clone(), &self.toml_path, &self.output_path)
            .await
            .unwrap();
        let file_contents = fs::read(proof_path).unwrap();
        let file_bytes = Bytes::from(file_contents);
        let proof_data = get_signed_proof(inputs.clone(), file_bytes).await.unwrap();
        return GenerateProofResponse {
            proof: proof_data.to_vec(),
        };
    }

    async fn benchmark(&self) -> BenchmarkResponse {
        let start_time = std::time::Instant::now();
        let public = vec![];
        let plain_secrets = vec![];
        let inputs = InputPayload::from_plain_secrets(public, plain_secrets);
        write_private_inputs_to_toml(inputs.clone(), &self.toml_path).unwrap();
        execute_prove_command(inputs, &self.toml_path, &self.output_path)
            .await
            .unwrap();
        let elapsed_time = start_time.elapsed().as_millis();
        
        BenchmarkResponse {
            data: "Success".to_string(),
            time_in_ms: elapsed_time as u128,
        }
    }
}

#[async_trait]
impl IVSTrait for NoirProver {
    async fn check_inputs(&self, _input: InputPayload) -> CheckInputResponse {
        CheckInputResponse { valid: true }
    }
    async fn check_inputs_and_proof(
        &self,
        _input: VerifyInputsAndProof,
    ) -> VerifyInputAndProofResponse {
        VerifyInputAndProofResponse {
            is_input_and_proof_valid: true,
        }
    }
}

use std::fs::File;
use std::io::Write;
use std::io::{Error, ErrorKind};
use std::process::{Command, Stdio};

async fn execute_prove_command(
    _inputs: InputPayload,
    toml_path: &str,
    output_path: &str,
) -> Result<String, Error> {
    let toml_file_path = format!("{}/{}.toml", toml_path, "temp");

    let witness = "foo";

    let mut cmd = Command::new("nargo");
    cmd.arg("execute")
        .arg("-p")
        .arg(&toml_file_path)
        .arg(&witness)
        .current_dir(toml_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    // // Execute the command asynchronously
    let output = cmd.output()?;
    // Check if the command was successful
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(Error::new(
            ErrorKind::Other,
            format!("Command failed: {}", stderr),
        ));
    }
    // bb prove -b ./target/hello_world.json -w ./target/witness-name.gz -o ./target/proof
    let output_file_path = output_path.to_string();
    let json_file_path = format!("{}/target/hello_world.json", toml_path);
    let witness_file_path = format!("{}/target/{}.gz", toml_path, witness);
    let mut cmd_bb = Command::new("bb");
    cmd_bb
        .arg("prove")
        .arg("-b")
        .arg(&json_file_path)
        .arg("-w")
        .arg(&witness_file_path)
        .arg("-o")
        .arg(&output_file_path)
        .current_dir(toml_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    // Execute the command asynchronously
    let output_bb = cmd_bb.output()?;
    // Check if the command was successful
    if !output_bb.status.success() {
        let stderr = String::from_utf8_lossy(&output_bb.stderr);
        return Err(Error::new(
            ErrorKind::Other,
            format!("Command failed: {}", stderr),
        ));
    }

    Ok(output_file_path)
}

use serde_json::Value;
fn write_private_inputs_to_toml(
    payload: InputPayload,
    toml_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // let ask_id = inputs.ask_id;
    // println!("{}",payload.get_plain_secrets().unwrap());
    let file_path = format!("{}/{}.toml", toml_path, "temp");

    // Convert Vec<u8> to String
    let json_string = String::from_utf8(payload.get_plain_secrets().unwrap())?;

    // Deserialize JSON string to serde_json::Value
    let json_value: Value = serde_json::from_str(&json_string)?;

    // Convert serde_json::Value to a TOML string
    let toml_string = toml::to_string(&json_value)?;

    // Write the TOML string to a file
    let mut file = File::create(file_path)?;
    file.write_all(toml_string.as_bytes())?;

    Ok(())
}

use ethers::prelude::*;
async fn get_signed_proof(
    inputs: InputPayload,
    proof: Bytes,
) -> Result<Bytes, Box<dyn std::error::Error>> {
    // Read the secp256k1 private key from file
    let read_secp_private_key = fs::read("/app/ecdsa.sec").expect("/app/ecdsa.sec file not found");
    let secp_private_key = secp256k1::SecretKey::from_slice(&read_secp_private_key)
        .expect("Failed reading secp_private_key get_signed_proof()")
        .display_secret()
        .to_string();
    let signer_wallet = secp_private_key
        .parse::<LocalWallet>()
        .expect("Failed creating signer_wallet get_signed_proof()");

    // Prepare the data for signing
    // let public_inputs = inputs.ask.prover_data.clone();
    let public_inputs: ethers::types::Bytes = inputs.clone().get_public().into();
    let proof_bytes = proof.clone();
    println!("{:?}", &proof_bytes);

    // Encode the data for signing
    let value = vec![
        ethers::abi::Token::Bytes(public_inputs.to_vec()),
        ethers::abi::Token::Bytes(proof_bytes.to_vec()),
    ];
    let encoded = ethers::abi::encode(&value);
    let digest = ethers::utils::keccak256(encoded);

    // Sign the message digest
    let signature = signer_wallet
        .sign_message(ethers::types::H256(digest))
        .await
        .expect("Failed creating signature get_signed_proof()");

    let sig_bytes: Bytes = signature.to_vec().into();
    // Encode the proof response
    let value = vec![
        ethers::abi::Token::Bytes(public_inputs.to_vec()),
        ethers::abi::Token::Bytes(proof_bytes.to_vec()),
        ethers::abi::Token::Bytes(sig_bytes.to_vec()),
    ];
    let encoded = ethers::abi::encode(&value);
    Ok(encoded.into())
}
