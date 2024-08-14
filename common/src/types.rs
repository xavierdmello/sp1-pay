use alloy_primitives::{U256, Bytes};
use alloy_sol_types::sol;
use serde::{Deserialize, Serialize};
use jwt_compact::jwk::JsonWebKey;

#[derive(Serialize, Deserialize)]
pub struct ProofInputs {
    pub identity_provider: U256,
    pub jwt: String,
    pub cert: Bytes,
}

// address msg_sender
// bytes32 claim_id
// bytes jsonwebkey
pub type ProofOutputs = sol! {
    tuple(address, bytes32, bytes)
};

#[derive(Deserialize, Serialize)]
pub struct JwkKeys {
    pub keys: Vec<ExtendedJsonWebKey<'static, Extra>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExtendedJsonWebKey<'a, T> {
    #[serde(flatten)]
    pub base: JsonWebKey<'a>,
    #[serde(flatten)]
    pub extra: T,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Extra {
    #[serde(rename = "kid")]
    pub key_id: String,
}