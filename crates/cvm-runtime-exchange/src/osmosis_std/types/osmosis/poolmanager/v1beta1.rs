use cvm_runtime::prelude::*;

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, Eq, prost::Message, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct MsgSwapExactAmountIn {
    #[prost(string, tag = "1")]
    pub sender: String,
    #[prost(message, repeated, tag = "2")]
    pub routes: Vec<SwapAmountInRoute>,
    #[prost(message, optional, tag = "3")]
    pub token_in: ::core::option::Option<ibc_apps_more::cosmos::Coin>,
    #[prost(string, tag = "4")]
    pub token_out_min_amount: String,
}

impl MsgSwapExactAmountIn {
    pub const TYPE_URL: &'static str = "/osmosis.poolmanager.v1beta1.MsgSwapExactAmountIn";
}

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, Eq, prost::Message, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct SwapAmountInRoute {
    #[prost(uint64, tag = "1")]
    #[serde(alias = "poolID")]
    pub pool_id: u64,
    #[prost(string, tag = "2")]
    pub token_out_denom: String,
}
