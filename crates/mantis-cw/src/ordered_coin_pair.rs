use cosmwasm_std::{Coin, Uint128};

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
            a: cosmwasm_std::Coin {
                denom: ab.a,
                ..Default::default()
            },
            b: cosmwasm_std::Coin {
                denom: ab.b,
                ..Default::default()
            },
        }
    }

    pub fn add_a(&mut self, amount: Uint128) {
        self.a.amount += amount;
    }

    pub fn add_b(&mut self, amount: Uint128) {
        self.b.amount += amount;
    }

    pub fn add(&mut self, coin: &Coin) {
        if coin.denom == self.a.denom {
            self.a.amount += coin.amount;
        } else if coin.denom == self.b.denom {
            self.b.amount += coin.amount;
        } else {
            panic!("invalid coin denom");
        }
    }
}

impl From<(Denom, Denom)> for OrderCoinPair {
    fn from(ab: (Denom, Denom)) -> Self {
        Self::zero(ab.0, ab.1)
    }
}
