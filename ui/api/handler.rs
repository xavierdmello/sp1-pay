use alloy_primitives::{Address, Bytes, FixedBytes, B256, U256};
use alloy_sol_types::{sol, SolInterface, SolType, SolValue};
use anyhow::Context;

use http_body_util::BodyExt;
use hyper::{body::Buf, Method};
use log::info;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sp1_sdk::{
    utils, HashableKey, ProverClient, SP1ProofWithPublicValues, SP1Stdin, SP1VerifyingKey,
};
use vercel_runtime::{run, Body, Error, Request, Response, StatusCode};
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

// address msg_sender
// bytes32 claim_id
pub type ProofOutputs = sol! {
    tuple(address, bytes32)
};

pub const ELF: &[u8] = include_bytes!("../../elf/riscv32im-succinct-zkvm-elf");

#[tokio::main]
async fn main() -> Result<(), Error> {
    run(handler).await
}

#[derive(Deserialize)]
struct TokenRequest {
    token: String,
}

// pub async fn handler(req: Request) -> Result<Response<Body>, Error> {
//     match (req.method(), req.uri().path()) {
//         (&Method::POST, "/prove") => {
//             // Read the body into bytes
//             let body = req.into_body().collect().await?.to_bytes();
//             let token_request: TokenRequest = serde_json::from_slice(&body)?;

//             match prove(token_request.token).await {
//                 Ok(calldata) => Ok(Response::builder()
//                     .status(StatusCode::OK)
//                     .header("Content-Type", "application/json")
//                     .body(json!({ "calldata": calldata }).to_string().into())?),
//                 Err(e) => Ok(Response::builder()
//                     .status(StatusCode::INTERNAL_SERVER_ERROR)
//                     .header("Content-Type", "application/json")
//                     .body(json!({ "error": e.to_string() }).to_string().into())?)
//             }
//         },
//         _ => Ok(Response::builder()
//             .status(StatusCode::NOT_FOUND)
//             .body("Not Found".into())?)
//     }
// }

pub async fn handler(_req: Request) -> Result<Response<Body>, Error> {
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(
            json!({
              "message": "你好，世界"
            })
            .to_string()
            .into(),
        )?)
}

// Prove the proof and return the calldata
pub async fn prove(token: String) -> Result<String, Error> {
    // Setup the logger.
    sp1_sdk::utils::setup_logger();
    // Setup the prover client.
    let client = ProverClient::new();

    // Setup the program.
    let (pk, vk) = client.setup(ELF);

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

    let claims = ProofOutputs::abi_decode(&proof.public_values.to_vec(), true)
        .context("decoding journal data")
        .expect("failed to decode");

    info!("Claim ID: {:?}", claims.0);
    info!("Msg Sender: {:?}", claims.1);
    let proof_as_bytes = if std::env::var("SP1_PROVER").unwrap().to_lowercase() == "mock" {
        vec![]
    } else {
        proof.bytes()
    };

    Ok(hex::encode(
        IBonsaiPay::IBonsaiPayCalls::claim(IBonsaiPay::claimCall {
            proof: Bytes::from(proof_as_bytes),
            publicValues: Bytes::from(proof.public_values.to_vec()),
        })
        .abi_encode(),
    ))
}
