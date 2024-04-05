use crate::{Denom, OrderedTuple2};

/// CosmWasm Coin pair ordered by denom
pub struct OrderCoinPair {
    pub a: cosmwasm_std::Coin,
    pub b: cosmwasm_std::Coin,
}

impl OrderCoinPair {
    pub fn zero(a: Denom, b: Denom) -> Self {
        let ab = OrderedTuple2::new(a, b);
        Self {
            a: cosmwasm_std::Coin { denom : ab.a, ..Default::default()},
            b: cosmwasm_std::Coin { denom : ab.b, ..Default::default()},
        }
    }
}

impl From<(Denom, Denom)> for OrderCoinPair {
    fn from(ab: (Denom, Denom)) -> Self {
        Self::zero(ab.0, ab.1)
    }
}