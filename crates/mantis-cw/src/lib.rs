mod ordered_coin_pair;
mod ordered_tuple;

pub use ordered_coin_pair::OrderCoinPair;
pub use ordered_tuple::OrderedTuple2;

pub type Denom = String;
pub type DenomPair = OrderedTuple2<String>;

/// this is buy sell in terms of token1/token2 or A/B. just 2 sides of the orderbook.
/// not Buy and Sell orders which differ in limit definition(in limit vs out limit).
#[derive(Debug, PartialEq, Eq, Clone, Copy, strum_macros::AsRefStr, derive_more::Display)]
#[cfg_attr(
    feature = "json-schema", // all(feature = "json-schema", not(target_arch = "wasm32")),
    derive(schemars::JsonSchema)
)]
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]

pub enum OrderType {
    Buy = 0,
    Sell = 1,
}
