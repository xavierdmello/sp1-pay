use alloy_primitives::{Address, Bytes, B256, U256};
use serde::{Deserialize, Serialize};
use alloy_sol_types::sol;
use alloy_sol_types::SolType;

#[derive(Serialize, Deserialize)]
pub struct ProofInputs {
    pub identity_provider: U256,
    pub jwt: String,
    pub cert: Bytes,
}

pub type ProofOutputs = sol! {
    tuple(address, bytes32, bytes)
};