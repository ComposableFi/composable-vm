/// given all orders amounts aggregated into common pool,
/// ensure that solution does not violates this pull
/// and return proper action to handle settling funds locally according solution
#[no_panic]
pub fn solves_cows_via_bank(
    all_orders: &Vec<SolvedOrder>,
    mut a_total_in: u128,
    mut b_total_in: u128,
) -> Result<Vec<CowFilledOrder>, StdError> {
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
    Ok(transfers)
}
