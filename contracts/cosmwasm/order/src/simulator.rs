use cosmwasm_std::Storage;
use cvm_runtime::shared::XcProgram;

use crate::prelude::*;
use crate::CowFilledOrder;
use crate::CowSolutionCalculation;
use crate::SolvedOrder;

/// given all orders amounts aggregated into common pool,
/// ensure that solution does not violates this pull
/// and return proper action to handle settling funds locally according solution
pub fn simulate_cows_via_bank(
    all_orders: &Vec<SolvedOrder>,
    mut a_total_in: u128,
    mut b_total_in: u128,
) -> Result<CowSolutionCalculation, StdError> {
    let mut transfers = vec![];
    for order in all_orders.iter() {
        let cowed = order.solution.cow_amount;
        let filled_wanted = Coin {
            amount: cowed,
            denom: order.wanted_denom().clone(),
        };

        // so if not enough was deposited as was taken from original orders, it will fail - so
        // solver cannot rob the bank
        if order.pair().0 == filled_wanted.denom {
            a_total_in = a_total_in.checked_sub(cowed.u128()).ok_or_else(|| {
                StdError::generic_err(format!("a underflow: {} {}", a_total_in, cowed.u128()))
            })?;
        } else {
            b_total_in = b_total_in.checked_sub(cowed.u128()).ok_or_else(|| {
                StdError::generic_err(format!("b underflow: {} {}", b_total_in, cowed.u128()))
            })?;
        };

        transfers.push((filled_wanted, order.order.order_id));
    }
    let result = CowSolutionCalculation {
        filled: transfers,
        token_a_remaining: a_total_in.into(),
        token_b_remaining: b_total_in.into(),
    };
    Ok(result)
}

/// Check that start and end either a/b or b/a on centauri.
/// And checks that amounts out (a or b) is more than remaining.
/// Solve only larger CVM for in volume, assuming other solution will be for other side sent.
/// Produces remaining each order will receive proportional to what is given.
pub fn simulate_route(
    storage: &mut dyn Storage,
    route: XcProgram,
    token_a_remaining: Coin,
    token_b_remaining: Coin,
    orders: Vec<SolvedOrder>,
) -> Result<(), StdError> {
    todo!()
}
