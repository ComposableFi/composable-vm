use core::num;

use cosmrs::tendermint::block::Height;
use cvm_runtime::{
    outpost::GetConfigResponse, shared::{CvmAddress, Displayed}, Amount, AssetId
};
use cw_mantis_order::{ordered_tuple::OrderedTuple2, CrossChainPart, Denom, DenomPair, OrderAmount, OrderItem, OrderSolution, OrderSubMsg};

use crate::{
    prelude::*,
    solver::{orderbook::OrderList, solution::Solution, types::OrderType},
};

use super::cosmos::client::timeout;

/// input batched summarized from users for routing
pub struct IntentBankInput {
    pub in_asset_id: AssetId,
    pub in_asset_amount: Displayed<u64>,
    pub out_asset_id: AssetId,
    pub order_accounts: Vec<(CvmAddress, Amount)>,
}



impl IntentBankInput {
    pub fn new(
        in_asset_id: AssetId,
        in_asset_amount: Displayed<u64>,
        out_asset_id: AssetId,
        order_accounts: Vec<(CvmAddress, Amount)>,
    ) -> Self {
        Self {
            in_asset_id,
            in_asset_amount,
            out_asset_id,
            order_accounts,
        }
    }

    /// given CoW solution and total amount of assets, aggregate remaining to bank for two sides
    pub fn find_intent_amount(cows: &[OrderSolution], orders: &[OrderItem], cvm_glt: &GetConfigResponse, pair: DenomPair ) -> (IntentBankInput, IntentBankInput) {
        
        for cow in cows {
            match cow.cross_chain_part {
                Some(OrderAmount::All) => {
                    let cowed = orders.iter().find(|x| x.order_id == cow.order_id).expect("order").clone();
                    cowed.
                },
                None => {},
                _ => panic!("unsupported cross chain part")
            }
        }
    }
}

pub type SolutionsPerPair = Vec<(Vec<OrderSolution>, (u64, u64))>;

pub fn find_cows(all_orders: Vec<OrderItem>) -> SolutionsPerPair {
    let all_orders = all_orders.into_iter().group_by(|x| {
        let mut ab = [x.given.denom.clone(), x.msg.wants.denom.clone()];
        ab.sort();
        (ab[0].clone(), ab[1].clone())
    });
    let mut cows_per_pair = vec![];
    for ((a, b), orders) in all_orders.into_iter() {
        let orders = orders.collect::<Vec<_>>();
        use crate::solver::cows::*;
        use crate::solver::types::*;
        let orders = orders.iter().map(|x| {
            let side = if x.given.denom == a {
                OrderType::Buy
            } else {
                OrderType::Sell
            };

            crate::solver::types::Order::new_integer(
                x.given.amount.u128(),
                x.msg.wants.amount.u128(),
                side,
                x.order_id,
            )
        });
        let orders = OrderList {
            value: orders.collect(),
        };
        orders.print();
        let optimal_price = orders.compute_optimal_price(1000);
        println!("optimal_price: {:?}", optimal_price);
        let mut solution = Solution::new(orders.value.clone());
        solution = solution.match_orders(optimal_price);
        solution.print();
        let cows = solution
            .orders
            .value
            .into_iter()
            .filter(|x| x.amount_out > <_>::default())
            .map(|x| {
                let filled = x.amount_out.to_u128().expect("u128");
                OrderSolution {
                    order_id: x.id,
                    cow_out_amount: filled.into(),
                    cross_chain_part: Some(OrderAmount::All),
                }
            })
            .collect::<Vec<_>>();
        println!("optimal price {:?}", optimal_price);
        let optimal_price = decimal_to_fraction(optimal_price.0);
        println!("cows: {:?}", cows);
        if !cows.is_empty() {
            cows_per_pair.push((cows, optimal_price));
        }
    }
    cows_per_pair
}

/// TODO: ditch decimals they are useless
/// convert decimal to normalized fraction
fn decimal_to_fraction(amount: Decimal) -> (u64, u64) {
    let decimal_string = amount.to_string();
    let decimal_string: Vec<_> = decimal_string.split('.').collect();
    if decimal_string.len() == 1 {
        (decimal_string[0].parse().expect("in range"), 1)
    } else {
        let digits_after_decimal = decimal_string[1].len() as u32;
        let denominator = 10_u128.pow(digits_after_decimal) as u64;
        let numerator = (amount * Decimal::from(denominator))
            .to_u64()
            .expect("integer");
        let fraction = fraction::Fraction::new(numerator, denominator);

        (
            *fraction.numer().expect("num"),
            *fraction.denom().expect("denom"),
        )
    }
}
