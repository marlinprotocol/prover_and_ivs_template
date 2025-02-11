# Confidential Provider & IVS Template

This repository provides a minimal template for running a **Confidential Prover** and an **IVS (Input Verification Service)** inside an enclave, along with a **Non-Confidential Prover** that can run anywhere. The provided code stubs demonstrate how to implement the necessary traits for proof generation, benchmarking, and attestation.

## Components

### Confidential Prover

The **Confidential Prover** is designed to run inside an enclave. It generates proofs and produces attestation while leveraging the security guarantees of the enclave. The sample implementation below uses a dummy `NullConfProver` that implements both the `GeneratorTrait` and `IVSTrait`.

#### Code Example

```rust
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
        // Actual logic here
        unimplemented!()
    }
}
```

#### Running the Confidential Prover

Once you have implemented the required logic, you can run the Confidential Prover with:

```bash
cargo run --release --bin confidential_prover
```

assuming that following ports are used in .env file
```
PROVER_PORT=3000
PROMETHEUS_PORT=9999
```

http://localhost:3000/swagger-ui/ and http://localhost:9999/swagger-ui/ should pop up successfully

> **Note:** Although this binary is executed outside the enclave, it will only be able to generate proofs and attestation inside the enclave environment.

---

### IVS (Input Verification Service)

The **IVS** component is responsible for verifying inputs and proofs. Like the Confidential Prover, the IVS is intended to run inside an enclave and generate attestation.

#### Code Example

```rust
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
        // Actual logic here
        unimplemented!()
    }
}
```

#### Running IVS

After implementing the necessary logic, run IVS with:

```bash
cargo run --release --bin ivs
```

assuming that following port is used in .env file
```
IVS_PORT=3000
```

http://localhost:3000/swagger-ui/

> **Note:** The IVS binary runs outside the enclave but will perform attestation only when operating inside the enclave.

---

### Non-Confidential Prover

The **Non-Confidential Prover** is a dummy implementation that does not require an enclave. It is designed to run anywhere and handles proof generation and benchmarking without the additional security of an enclave.

#### Code Example

```rust
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
```

#### Running the Non-Confidential Prover

To run the Non-Confidential Prover, use:

```bash
cargo run --release --bin non_confidential_prover
```

This prover does not require an enclave and can be executed in any environment.

---

## Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (ensure you have Rust installed)
- Any additional enclave dependencies required for your implementation

### Cloning the Repository

Clone the repository and navigate into the project directory:

```bash
git clone https://github.com/marlinprotocol/prover_and_ivs_template.git
cd prover_and_ivs_template
```

### Building and Running

Build and run any of the binaries using Cargo. For example, to run the Confidential Prover:

```bash
cargo run --release --bin confidential_prover
```

assuming that following ports are used in .env file
```
PROVER_PORT=3000
PROMETHEUS_PORT=9999
```

http://localhost:3000/swagger-ui/ and http://localhost:9999/swagger-ui/ should pop up successfully