use alloy_primitives::{Address, Bytes, FixedBytes, B256, U256};
use alloy_sol_types::{sol, SolInterface, SolType, SolValue};
use anyhow::Context;
use http_body_util::BodyExt;
use hyper::{body::Buf, Method};
use log::info;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sp1_sdk::{
    proto::network::ProofMode, utils, HashableKey, NetworkProver, PlonkBn254Proof, ProverClient, SP1ProofWithPublicValues, SP1Stdin, SP1VerifyingKey
};
use std::time::Duration;
use vercel_runtime::{run, Body, Error, Request, Response, StatusCode};
use dotenv::dotenv;

sol! {
    interface IBonsaiPay {
        function claim(bytes calldata proof, bytes calldata publicValues);
    }
}

#[derive(Serialize, Deserialize)]
pub struct ProofInputs {
    pub identity_provider: U256,
    pub jwt: String,
}

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

    match request_proof(token_request.jwt).await {
        Ok(proof_id) => Ok(Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "application/json")
            .body(Body::Text(json!({ "proofId": proof_id }).to_string()))?),
        Err(e) => Ok(Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .header("Content-Type", "application/json")
            .body(Body::Text(json!({ "error": e.to_string() }).to_string()))?)
    }
}

pub async fn request_proof(token: String) -> Result<String, Error> {
    // Setup the logger.
    sp1_sdk::utils::setup_logger();

    // Setup the inputs.
    let mut stdin = SP1Stdin::new();

    let inputs = ProofInputs {
        identity_provider: U256::ZERO,
        jwt: token,
    };
    stdin.write(&inputs);

    // Request the proof.
    let network_prover = NetworkProver::new();
    let proof_id = network_prover.request_proof(ELF, stdin, ProofMode::Plonk).await?;
    println!("Proof Link: https://explorer.succinct.xyz/proof/{}", proof_id);

    Ok(proof_id)
}