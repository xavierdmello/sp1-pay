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
    println!("YOOOO");
    let inputs: ProofInputs = sp1_zkvm::io::read();

    let identity_provider: IdentityProvider = inputs.identity_provider.into();
    let jwt: String = inputs.jwt;

    println!("jwt: {:?}", jwt);
    println!("identity_provider: {:?}", identity_provider);


    let (claim_id, msg_sender) = identity_provider.validate(&jwt).unwrap();
    println!("claim_id: {:?}", claim_id);
    println!("msg_sender: {:?}", msg_sender);
    let msg_sender: Address = Address::parse_checksummed(msg_sender, None).unwrap();
    println!("msg_sender: {:?}", msg_sender);
    let claim_id: FixedBytes<32> =
        FixedBytes::from_slice(Sha256::digest(claim_id.as_bytes()).as_slice());
    println!("claim_id: {:?}", claim_id);
    let output = ProofOutputs::abi_encode(&(msg_sender, claim_id));
    println!("output: {:?}", output);

    sp1_zkvm::io::commit_slice(&output);
}
