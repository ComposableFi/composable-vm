use crate::prelude::*;
use cvm::{exchange::ExchangeId, NetworkId};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[cfg_attr(
    feature = "json-schema", // all(feature = "json-schema", not(target_arch = "wasm32")),
    derive(schemars::JsonSchema)
)]
#[serde(rename_all = "snake_case")]
pub enum ExchangeType {
    #[cfg(all(feature = "cosmwasm", feature = "cosmos"))]
    OsmosisPoolManagerModuleV1Beta1 {
        pool_id: u64,
        token_a: String,
        token_b: String,
    },
    #[cfg(feature = "cosmwasm")]
    AstroportRouterContract {
        address: cosmwasm_std::Addr,
        token_a: String,
        token_b: String,
    },
}

/// allows to execute Exchange instruction
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[cfg_attr(
    feature = "json-schema", // all(feature = "json-schema", not(target_arch = "wasm32")),
    derive(schemars::JsonSchema)
)]
#[serde(rename_all = "snake_case")]
pub struct ExchangeItem {
    pub exchange_id: ExchangeId,
    pub network_id: NetworkId,
    pub exchange: ExchangeType,
    // if was set, means that cannot exchange via this venue
    pub closed: Option<u64>,
}

impl ExchangeItem {
    pub fn new(exchange_id: ExchangeId, network_id: NetworkId, exchange: ExchangeType) -> Self {
        Self {
            exchange_id,
            network_id,
            exchange,
            closed: None,
        }
    }
}
