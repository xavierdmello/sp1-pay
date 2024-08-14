use anyhow::Context;
use http_body_util::BodyExt;
use serde::Deserialize;
use serde_json::{json, Value};
use sp1_sdk::{
    NetworkProver, SP1ProofWithPublicValues
};
use std::time::Duration;
use vercel_runtime::{run, Body, Error, Request, Response, StatusCode};
use dotenv::dotenv;

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenv().ok();
    run(handler).await
}

#[derive(Deserialize)]
struct ProofIdRequest {
    proof_id: String,
}

pub async fn handler(req: Request) -> Result<Response<Body>, Error> {
    let body = req.into_body().collect().await?;
    let bytes = body.to_bytes();

    let proof_id_request: ProofIdRequest = serde_json::from_slice(&bytes)
        .context("Failed to deserialize request body")?;

    match wait_proof(&proof_id_request.proof_id).await {
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

pub async fn wait_proof(proof_id: &str) -> Result<Value, Error> {
    let network_prover = NetworkProver::new();
    let proof = network_prover.wait_proof::<SP1ProofWithPublicValues>(proof_id, Some(Duration::from_secs(240))).await?;
    let proof_bytes = proof.bytes();

    Ok(json!({
        "proof": format!("0x{}", hex::encode(proof_bytes)),
        "publicValues": format!("0x{}", hex::encode(proof.public_values.to_vec())),
        "proofId": proof_id
    }))
}