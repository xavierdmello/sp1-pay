// Copyright 2024 RISC Zero, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use alloy_primitives::{FixedBytes, U256};
use alloy_sol_types::{sol, SolInterface, SolValue};
use anyhow::Context;
use clap::Parser;
use fibonacci_script::TxSender;
use log::info;
use sp1_sdk::{HashableKey, ProverClient, SP1ProofWithPublicValues, SP1Stdin, SP1VerifyingKey};
use tokio::sync::oneshot;
use warp::Filter;
use serde::{Deserialize, Serialize};
pub const FIBONACCI_ELF: &[u8] = include_bytes!("../../../program/elf/riscv32im-succinct-zkvm-elf");

sol! {
    interface IBonsaiPay {
        function claim(address payable to, bytes32 claim_id, bytes32 post_state_digest, bytes calldata seal);
    }

    struct ClaimsData {
        address msg_sender;
        bytes32 claim_id;
    }
}

// uint256 identity_provider
// string jwt
type InputTuple = sol! {
    tuple(uint256, string)
};

/// Arguments of the publisher CLI.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Ethereum chain ID
    #[clap(long)]
    chain_id: u64,

    /// Ethereum Node endpoint.
    #[clap(long, env)]
    eth_wallet_private_key: String,

    /// Ethereum Node endpoint.
    #[clap(long)]
    rpc_url: String,

    /// Application's contract address on Ethereum
    #[clap(long)]
    contract: String,
}

const HEADER_XAUTH: &str = "X-Auth-Token";

async fn handle_jwt_authentication(token: String) -> Result<(), warp::Rejection> {
    if token.is_empty() {
        return Err(warp::reject::reject());
    }

    info!("Token received: {}", token);

    let args = Args::parse();
    let (tx, rx) = oneshot::channel();

    // Spawn a new thread for the Bonsai Prover computation
    std::thread::spawn(move || {
        prove_and_send_transaction(args, token, tx);
    });

    match rx.await {
        Ok(_result) => Ok(()),
        Err(_) => Err(warp::reject::reject()),
    }
}

fn prove_and_send_transaction(
    args: Args,
    token: String,
    tx: oneshot::Sender<(Vec<u8>, FixedBytes<32>, Vec<u8>)>,
) {
    dotenv::dotenv().ok();

    // Setup the logger.
    sp1_sdk::utils::setup_logger();

    // Parse the command line arguments.
    let args = ProveArgs::parse();

    // Setup the prover client.
    let client = ProverClient::new();

    // Setup the program.
    let (pk, vk) = client.setup(FIBONACCI_ELF);

    // Setup the inputs.
    let mut stdin = SP1Stdin::new();

    let input = Input {
        identity_provider: U256::ZERO, // Google as the identity provider
        jwt: token,
    };

    stdin.write_slice(&input.abi_encode());
    // Generate the proof.
    let proof = client
        .prove(&pk, stdin)
        .plonk()
        .run()
        .expect("failed to generate proof");
    create_plonk_fixture(&proof, &vk);

    let seal_clone = seal.clone();

    let tx_sender = TxSender::new(
        args.chain_id,
        &args.rpc_url,
        &args.eth_wallet_private_key,
        &args.contract,
    )
    .expect("failed to create tx sender");

    let claims = ClaimsData::abi_decode(&journal, true)
        .context("decoding journal data")
        .expect("failed to decode");

    info!("Claim ID: {:?}", claims.claim_id);
    info!("Msg Sender: {:?}", claims.msg_sender);

    let calldata = IBonsaiPay::IBonsaiPayCalls::claim(IBonsaiPay::claimCall {
        to: claims.msg_sender,
        claim_id: claims.claim_id,
        post_state_digest,
        seal: seal_clone,
    })
    .abi_encode();

    // Send the calldata to Ethereum.
    let runtime = tokio::runtime::Runtime::new().expect("failed to start new tokio runtime");
    runtime
        .block_on(tx_sender.send(calldata))
        .expect("failed to send tx");

    tx.send((journal, post_state_digest, seal))
        .expect("failed to send over channel");
}

fn jwt_authentication_filter() -> impl Filter<Extract = ((),), Error = warp::Rejection> + Clone {
    warp::any()
        .and(warp::header::<String>(HEADER_XAUTH))
        .and_then(handle_jwt_authentication)
}

fn auth_filter() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let cors = warp::cors()
        .allow_any_origin()
        .allow_methods(vec!["GET", "POST", "DELETE"])
        .allow_headers(vec!["content-type", "x-auth-token"])
        .max_age(3600);

    warp::path("auth")
        .and(warp::get())
        .and(warp::path::end())
        .and(jwt_authentication_filter().untuple_one())
        .map(|| warp::reply())
        .with(cors)
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let api = auth_filter();

    warp::serve(api).run(([127, 0, 0, 1], 8080)).await;
}


/// A fixture that can be used to test the verification of SP1 zkVM proofs inside Solidity.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SP1FibonacciProofFixture {
    a: u32,
    b: u32,
    n: u32,
    vkey: String,
    public_values: String,
    proof: String,
}

/// Create a fixture for the given proof.
fn create_plonk_fixture(proof: &SP1ProofWithPublicValues, vk: &SP1VerifyingKey) {
    // Deserialize the public values.
    let bytes = proof.public_values.as_slice();
    let (n, a, b) = PublicValuesTuple::abi_decode(bytes, false).unwrap();

    // Create the testing fixture so we can test things end-ot-end.
    let fixture = SP1FibonacciProofFixture {
        a,
        b,
        n,
        vkey: vk.bytes32().to_string(),
        public_values: format!("0x{}", hex::encode(bytes)),
        proof: format!("0x{}", hex::encode(proof.bytes())),
    };

    // The verification key is used to verify that the proof corresponds to the execution of the
    // program on the given input.
    //
    // Note that the verification key stays the same regardless of the input.
    println!("Verification Key: {}", fixture.vkey);

    // The public values are the values whicha are publically commited to by the zkVM.
    //
    // If you need to expose the inputs or outputs of your program, you should commit them in
    // the public values.
    println!("Public Values: {}", fixture.public_values);

    // The proof proves to the verifier that the program was executed with some inputs that led to
    // the give public values.
    println!("Proof Bytes: {}", fixture.proof);

    // Save the fixture to a file.
    let fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../contracts/src/fixtures");
    std::fs::create_dir_all(&fixture_path).expect("failed to create fixture path");
    std::fs::write(
        fixture_path.join("fixture.json"),
        serde_json::to_string_pretty(&fixture).unwrap(),
    )
    .expect("failed to write fixture");
}
