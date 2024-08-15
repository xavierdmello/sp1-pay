use alloy_primitives::{Address, B256};
use anyhow::Context;
use http_body_util::BodyExt;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value };
use sp1_sdk::{
    NetworkProver, SP1ProofWithPublicValues, SP1VerifyingKey, HashableKey
};
use std::time::Duration;
use vercel_runtime::{run, Body, Error, Request, Response, StatusCode};
use dotenv::dotenv;
use std::path::PathBuf;
use ui::ProofInputs;
use ui::ProofOutputs;
use alloy_sol_types::SolType;

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
    let proof = network_prover.wait_proof::<SP1ProofWithPublicValues>(proof_id, Some(Duration::from_secs(300))).await?;
    let proof_bytes = proof.bytes();

    create_plonk_fixture(&proof);
    
    Ok(json!({
        "proof": format!("0x{}", hex::encode(proof_bytes)),
        "publicValues": format!("0x{}", hex::encode(proof.public_values.to_vec())),
        "proofId": proof_id
    }))
}

/// A fixture that can be used to test the verification of SP1 zkVM proofs inside Solidity.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SP1FibonacciProofFixture {
    msg_sender: Address,
    claim_id: B256,
    public_values: String,
    proof: String,
}

/// Create a fixture for the given proof.
fn create_plonk_fixture(proof: &SP1ProofWithPublicValues) {
    // Deserialize the public values.
    let bytes = proof.public_values.as_slice();
    let (msg_sender, claim_id, _) = ProofOutputs::abi_decode(bytes, false).unwrap();

    // Create the testing fixture so we can test things end-to-end.
    let fixture = SP1FibonacciProofFixture {
        msg_sender,
        claim_id,
        public_values: format!("0x{}", hex::encode(bytes)),
        proof: format!("0x{}", hex::encode(proof.bytes())),
    };

    // The public values are the values whicha are publically commited to by the zkVM.
    //
    // If you need to expose the inputs or outputs of your program, you should commit them in
    // the public values.
    println!("Public Values: {}", fixture.public_values);

    // The proof proves to the verifier that the program was executed with some inputs that led to
    // the give public values.
    println!("Proof Bytes: {}", fixture.proof);

    // Save the fixture to a file.
    let fixture_path = PathBuf::from("./");
    std::fs::create_dir_all(&fixture_path).expect("failed to create fixture path");
    std::fs::write(
        fixture_path.join("fixture.json"),
        serde_json::to_string_pretty(&fixture).unwrap(),
    )
    .expect("failed to write fixture");
}