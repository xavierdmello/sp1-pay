//! A simple program that takes a number `n` as input, and writes the `n-1`th and `n`th fibonacci
//! number as an output.

// These two lines are necessary for the program to properly compile.
//
// Under the hood, we wrap your main function with some extra code so that it behaves properly
// inside the zkVM.
#![no_main]
sp1_zkvm::entrypoint!(main);

use alloy_primitives::{Address, FixedBytes};
use alloy_sol_types::SolType;
use common::{ProofInputs, ProofOutputs};
use oidc_validator::IdentityProvider;
use sha2::{Digest, Sha256};
fn main() {
    println!("cycle-tracker-start: read input");
    let inputs: ProofInputs = sp1_zkvm::io::read();
    println!("cycle-tracker-end: read input");

    println!("cycle-tracker-start: identity provider");
    let identity_provider: IdentityProvider = inputs.identity_provider.into();
    println!("cycle-tracker-end: identity provider");

    println!("cycle-tracker-start: jwt");
    let jwt: String = inputs.jwt;
    println!("cycle-tracker-end: jwt");
    println!("cycle-tracker-start: validate");
    let (claim_id, msg_sender) = identity_provider.validate(&jwt).unwrap();
    println!("cycle-tracker-end: validate");

    println!("cycle-tracker-start: msg sender");
    let msg_sender: Address = Address::parse_checksummed(msg_sender, None).unwrap();
    println!("cycle-tracker-end: msg sender");

    println!("cycle-tracker-start: claim id");
    let claim_id: FixedBytes<32> =
        FixedBytes::from_slice(Sha256::digest(claim_id.as_bytes()).as_slice());
    let output = ProofOutputs::abi_encode(&(msg_sender, claim_id));
    println!("cycle-tracker-end: claim id");

    println!("cycle-tracker-start: commit");
    sp1_zkvm::io::commit_slice(&output);
    println!("cycle-tracker-end: commit");
}
