use alloy_primitives::{Address, Bytes, FixedBytes, B256, U256};
use alloy_sol_types::{sol, SolInterface, SolType, SolValue};
use anyhow::Context;

use http_body_util::BodyExt;
use hyper::{body::Buf, Method};
use log::info;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sp1_sdk::{
    utils, HashableKey, ProverClient, SP1ProofWithPublicValues, SP1Stdin, SP1VerifyingKey,
};
use vercel_runtime::{run, Body, Error, Request, Response, StatusCode};
sol! {
    interface IBonsaiPay {
        function claim(bytes calldata proof, bytes calldata publicValues);
    }
}
use dotenv::dotenv;

#[derive(Serialize, Deserialize)]
pub struct ProofInputs {
    pub identity_provider: U256,
    pub jwt: String,
}

// address msg_sender
// bytes32 claim_id
pub type ProofOutputs = sol! {
    tuple(address, bytes32)
};

pub const ELF: &[u8] = include_bytes!("../../elf/riscv32im-succinct-zkvm-elf");

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenv().ok();
    run(handler).await
}

#[derive(Deserialize)]
struct TokenRequest {
    jwt: String,
}

pub async fn handler(req: Request) -> Result<Response<Body>, Error> {
    let body = req.into_body().collect().await?;
    let bytes = body.to_bytes();

    let token_request: TokenRequest = serde_json::from_slice(&bytes)
        .context("Failed to deserialize request body")?;

    match prove(token_request.jwt).await {
        Ok(json) => Ok(Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "application/json")
            .body(Body::Text(json.to_string()))?),
        Err(e) => Ok(Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .header("Content-Type", "application/json")
            .body(Body::Text(json!({ "error": e.to_string() }).to_string()))?)
    }
}

// Prove the proof and return the calldata
pub async fn prove(token: String) -> Result<Value, Error> {
    // Setup the logger.
    sp1_sdk::utils::setup_logger();
    // Setup the prover client.
    let client = ProverClient::new();

    // Setup the program.
    let (pk, _) = client.setup(ELF);

    // Setup the inputs.
    let mut stdin = SP1Stdin::new();

    let inputs = ProofInputs {
        identity_provider: U256::ZERO,
        jwt: token,
    };
    stdin.write(&inputs);

    // Generate the proof.
    let proof = client
        .prove(&pk, stdin)
        .plonk()
        .run()
        .expect("failed to generate proof");

    let proof_bytes = if std::env::var("SP1_PROVER").unwrap().to_lowercase() == "mock" {
        vec![]
    } else {
        proof.bytes()
    };
    
    Ok(json!({
        "proof": format!("0x{}", hex::encode(proof_bytes)),
        "publicValues": format!("0x{}", hex::encode(proof.public_values.to_vec()))
    }))
}