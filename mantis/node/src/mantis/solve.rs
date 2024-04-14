use cvm_runtime::{
    outpost::GetConfigResponse,
    shared::{CvmAddress, CvmBalanceFilter, CvmFundsFilter, CvmInstruction, Displayed},
    AssetId, Destination,
};
use cw_mantis_order::{OrderAmount, OrderItem, OrderSolution};
use mantis_cw::{DenomPair, OrderCoinPair};
use num_rational::Ratio;

use crate::{
    prelude::*,
    solver::cows::{orderbook::OrderList, solution::Solution},
};

/// input batched summarized from users for routing
#[derive(Debug, Clone)]
pub struct IntentBankInput {
    pub in_asset_id: AssetId,
    pub in_asset_amount: Displayed<u128>,
    pub out_asset_id: AssetId,
    pub order_accounts: Vec<CvmInstruction>,
}

impl IntentBankInput {
    pub fn new(
        in_asset_id: AssetId,
        in_asset_amount: Displayed<u128>,
        out_asset_id: AssetId,
        order_accounts: Vec<CvmInstruction>,
    ) -> Self {
        Self {
            in_asset_id,
            in_asset_amount,
            out_asset_id,
            order_accounts,
        }
    }

    /// given CoW solution and total amount of assets, aggregate remaining to bank for two sides
    pub fn find_intent_amount(
        cows: &[OrderSolution],
        orders: &[OrderItem],
        optimal_ratio: Ratio<u64>,
        cvm_glt: &GetConfigResponse,
        pair: DenomPair,
    ) -> Vec<IntentBankInput> {
        // native calculations
        let mut pair = OrderCoinPair::zero(pair.a, pair.b);
        let mut a_to_b = Vec::new();
        let mut b_to_a = Vec::new();

        for cow in cows {
            match cow.cross_chain_part {
                Some(OrderAmount::AllRemaining) => {
                    let mut order = orders
                        .iter()
                        .find(|x| x.order_id == cow.order_id)
                        .expect("order")
                        .clone();
                    order
                        .fill(cow.cow_out_amount, optimal_ratio.into())
                        .expect("off chain");
                    pair.add(&order.given);

                    if order.given.denom == pair.a.denom {
                        a_to_b.push((order.owner, order.given.amount));
                    } else {
                        b_to_a.push((order.owner, order.given.amount));
                    }
                }
                None => {}
                _ => panic!("unsupported cross chain part"),
            }
        }

        // making MANTIS route request in CVM form

        let mut intents = vec![];
        let a_asset = cvm_glt.cvm_asset_by_cw(pair.a.denom);
        let b_asset = cvm_glt.cvm_asset_by_cw(pair.b.denom);
        
        if pair.a.amount.u128() > 0 {
            let b_received = a_to_b.iter().map(|x| {
                let part = Ratio::new(x.1.u128(), pair.a.amount.u128()).msb_limit_unsigned();
                let part = CvmBalanceFilter::from((*part.numer(), *part.denom()));
                CvmInstruction::Transfer {
                    to: Destination::Account(CvmAddress::from(x.0.to_string())),
                    assets: CvmFundsFilter::of(b_asset, part),
                }
            });
            let intent =
                IntentBankInput::new(a_asset, pair.a.amount.into(), b_asset, b_received.collect());
            intents.push(intent);
        }

        if pair.b.amount.u128() > 0 {
            let a_received = b_to_a.iter().map(|x| {
                let part = Ratio::new(x.1.u128(), pair.b.amount.u128()).msb_limit_unsigned();
                let part = CvmBalanceFilter::from((*part.numer(), *part.denom()));
                CvmInstruction::Transfer {
                    to: Destination::Account(CvmAddress::from(x.0.to_string())),
                    assets: CvmFundsFilter::of(a_asset, part),
                }
            });
            let intent =
                IntentBankInput::new(b_asset, pair.b.amount.into(), a_asset, a_received.collect());
            intents.push(intent);
        }
        intents
    }
}

#[derive(Debug, Clone)]
pub struct PairSolution {
    pub ab: DenomPair,
    pub cows: Vec<OrderSolution>,
    pub optimal_price: Ratio<u64>,
}

/// seems right to do:
/// if cross chain enabled, it must do next:
/// 1. solve optimal price for existing cows.
/// 2. iterate over router and amounts to find good routing `cow`s
/// 3. return cow optimal price with cross chain (virtual) order.
pub fn find_cows(all_orders: &[OrderItem]) -> Vec<PairSolution> {
    let all_orders = all_orders.into_iter().group_by(|x| x.pair());
    let mut cows_per_pair = vec![];
    for (ab, orders) in all_orders.into_iter() {
        let orders = orders.collect::<Vec<_>>();
        let orders = orders.iter().map(|x| {
            crate::solver::types::SolverOrder::new_integer(
                x.given.amount.u128(),
                x.msg.wants.amount.u128(),
                x.side(&ab),
                x.order_id,
            )
        });
        let orders = OrderList {
            value: orders.collect(),
        };
        let optimal_price = orders.compute_optimal_price(1000);
        println!("mantis::solver::cows::optimal_price: {:?}", optimal_price);
        println!("mantis::solver::cows::orders: {:?}", orders);
        let mut solution = Solution::new(orders.value.clone());
        solution = solution.match_orders(optimal_price);
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
                    cross_chain_part: Some(OrderAmount::AllRemaining),
                }
            })
            .collect::<Vec<_>>();
        let optimal_price = decimal_to_fraction(optimal_price.0);
        println!("cows: {:?}", cows);
        if !cows.is_empty() {
            let pair_solution = PairSolution {
                ab,
                cows,
                optimal_price,
            };
            cows_per_pair.push(pair_solution);
        }
    }
    cows_per_pair
}

/// TODO: ditch decimals they are useless
/// convert decimal to normalized fraction
fn decimal_to_fraction(amount: Decimal) -> Ratio<u64> {
    let decimal_string = amount.to_string();
    let decimal_string: Vec<_> = decimal_string.split('.').collect();
    if decimal_string.len() == 1 {
        Ratio::new(decimal_string[0].parse().expect("in range"), 1)
    } else {
        let digits_after_decimal = decimal_string[1].len() as u32;
        let denominator = 10_u128.pow(digits_after_decimal) as u64;
        let numerator = (amount * Decimal::from(denominator))
            .to_u64()
            .expect("integer");
        Ratio::new(numerator, denominator)
    }
}
