use alloy_primitives::U256;
use alloy_sol_types::sol;
use serde::{Deserialize, Serialize};

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
