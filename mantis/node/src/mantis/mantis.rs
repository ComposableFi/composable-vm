use cw_mantis_order::{OrderItem, OrderSolution};

use crate::{
    prelude::*,
    solver::{orderbook::OrderList, solution::Solution, types::OrderType},
};

pub type SolutionsPerPair = Vec<(Vec<OrderSolution>, (u64, u64))>;

pub fn do_cows(all_orders: Vec<OrderItem>) -> SolutionsPerPair {
    let all_orders = all_orders.into_iter().group_by(|x| {
        let mut ab = [x.given.denom.clone(), x.msg.wants.denom.clone()];
        ab.sort();
        (ab[0].clone(), ab[1].clone())
    });
    let mut cows_per_pair = vec![];
    for ((a, b), orders) in all_orders.into_iter() {
        let orders = orders.collect::<Vec<_>>();
        use crate::solver::solver::*;
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
                    cow_amount: filled.into(),
                    cross_chain: 0u128.into(),
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
