//! A simple program that takes a number `n` as input, and writes the `n-1`th and `n`th fibonacci
//! number as an output.

// These two lines are necessary for the program to properly compile.
//
// Under the hood, we wrap your main function with some extra code so that it behaves properly
// inside the zkVM.
#![no_main]
sp1_zkvm::entrypoint!(main);

use alloy_primitives::{Address, FixedBytes};
use alloy_sol_types::SolValue;
use oidc_validator::IdentityProvider;
use sha2::{Sha256, Digest};

alloy_sol_types::sol! {
    struct ClaimsData {
        address msg_sender;
        bytes32 claim_id;
    }
    struct Input {
        uint256 identity_provider;
        string jwt;
    }
}

fn main() {
    let mut input_bytes = sp1_zkvm::io::read_vec();
    let input: Input = <Input>::abi_decode(&input_bytes, true).unwrap();

    let identity_provider: IdentityProvider = input.identity_provider.into();
    let jwt: String = input.jwt;

    let (claim_id, msg_sender) = identity_provider.validate(&jwt).unwrap();
    let msg_sender: Address = Address::parse_checksummed(msg_sender, None).unwrap();
    let claim_id: FixedBytes<32> =
        FixedBytes::from_slice(Sha256::digest(claim_id.as_bytes()).as_slice());
    let output = ClaimsData {
        msg_sender,
        claim_id,
    };
    let output = output.abi_encode();

    sp1_zkvm::io::commit_slice(&output);
}
