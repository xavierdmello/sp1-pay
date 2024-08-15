use alloy_primitives::{U256, Bytes};
use alloy_sol_types::sol;
use anyhow::Context;
use http_body_util::BodyExt;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sp1_sdk::{
    proto::network::ProofMode, NetworkProver, SP1Stdin
};
use vercel_runtime::{run, Body, Error, Request, Response, StatusCode};
use dotenv::dotenv;
use reqwest::get;
use std::str::FromStr;
use serde_json::Value;
use ui::ProofInputs;
use ui::ProofOutputs;
sol! {
    interface IBonsaiPay {
        function claim(bytes calldata proof, bytes calldata publicValues);
    }
}

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
        cert: fetch_google_jwt_cert().await.unwrap(),
    };
    stdin.write(&inputs);

    // Request the proof.
    let network_prover = NetworkProver::new();
    let proof_id = network_prover.request_proof(ELF, stdin, ProofMode::Plonk).await?;
    println!("Proof Link: https://explorer.succinct.xyz/proof/{}", proof_id);

    Ok(proof_id)
}

pub async fn fetch_google_jwt_cert() -> Result<Bytes, Box<dyn std::error::Error>> {
    let url = "https://www.googleapis.com/oauth2/v3/certs";
    let response = get(url).await?.json::<serde_json::Value>().await?;
    let bytes = Bytes::from(serde_json::to_vec(&response).unwrap());

    Ok(bytes)
}

