//! solana_program types until they support serde/json-schema

use core::fmt::Display;

use serde::{Deserialize, Serialize};

/// Is `solana-program` crate `Pubkey` type, but with proper serde support into base58 encoding.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(
    feature = "json-schema", // all(feature = "json-schema", not(target_arch = "wasm32")),
    derive(schemars::JsonSchema)
)]
pub struct Pubkey(pub(crate) [u8; 32]);
