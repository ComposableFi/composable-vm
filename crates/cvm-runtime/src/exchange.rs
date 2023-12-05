use crate::{prelude::*, NetworkId};

pub type ExchangeId = crate::shared::Displayed<u128>;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[cfg_attr(
    all(feature = "json-schema", not(target_arch = "wasm32")),
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
        address: Addr,
        token_a: String,
        token_b: String,
    },
}

/// allows to execute Exchange instruction
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[cfg_attr(
    all(feature = "json-schema", not(target_arch = "wasm32")),
    derive(schemars::JsonSchema)
)]
#[serde(rename_all = "snake_case")]
pub struct ExchangeItem {
    pub exchange_id: ExchangeId,
    pub network_id: NetworkId,
    pub exchange: ExchangeType,
}
