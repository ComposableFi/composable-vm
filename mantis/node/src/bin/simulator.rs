

use mantis_node::solver::types::OrderType;
use mantis_node::solver::{orderbook::OrderList, solution::Solution, types::Order};
use mantis_node::{prelude::*, solver::types::Price};

fn main() {
    // decide on basics
    let order_a = Order::new_integer(100_000, 3, OrderType::Sell, 1);
    let order_b = Order::new_integer(3, 100_000, OrderType::Buy, 2);
    assert!(order_a.is_acceptable_price(order_b.limit_price));
    assert!(order_b.is_acceptable_price(order_a.limit_price));

    let ab = OrderList {
        value: vec![order_a, order_b],
    };
    ab.print();
    let optimal_price = ab.compute_optimal_price(50);

    let mut solution = Solution::new(ab.value.clone());
    solution = solution.match_orders(optimal_price);
    solution.print();

    println!(
        "{:}",
        solution.orders.value[0]
            .amount_filled
            .to_u128()
            .expect("msg")
    );
    println!(
        "{:}",
        solution.orders.value[1]
            .amount_filled
            .to_u128()
            .expect("msg")
    );

    // randomize price around 2.0 (ratio of 2 price tokens in pair)
    let orders = (1..100).map(|x| Order::random_f64(2., 0.1, (50, 150), x));
    let orders = OrderList {
        value: orders.collect(),
    };
    orders.print();

    // solves nothing as no really overlap of orders
    let mut solution = Solution::new(orders.value.clone());
    solution = solution.match_orders(Price::new_float(1.0));
    solution.print();

    // solves for some specific price some
    let mut solution = Solution::new(orders.value.clone());
    solution = solution.match_orders(Price::new_float(2.05));
    solution.print();

    // finds maximal volume price
    let mut solution = Solution::new(orders.value.clone());
    solution = solution.match_orders(optimal_price);
    solution.print();

    // big simulation
    let optimal_price = orders.compute_optimal_price(50);

    // fill orders
    let mut solution = Solution::new(orders.value.clone());
    solution = solution.match_orders(optimal_price);
    solution.print();
}
