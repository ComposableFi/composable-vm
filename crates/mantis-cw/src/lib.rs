mod ordered_coin_pair;
mod ordered_tuple;

pub use ordered_coin_pair::OrderCoinPair;
pub use ordered_tuple::OrderedTuple2;
use rust_decimal::Decimal;

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

pub enum OrderSide {
    A = 0,
    B = 1,
}


impl OrderSide {
    // ddo not use decimal, decimal is bad
    pub fn is_acceptable_price(&self, price: Decimal, limit_price: Decimal) -> bool {
        match self {
            OrderSide::B => price >= limit_price,
            OrderSide::A => price <= limit_price,
        }
    }
}