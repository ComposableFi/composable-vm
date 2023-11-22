use bip32::secp256k1::elliptic_curve::rand_core::block;
use cosmos_sdk_proto::{
    cosmos::{auth::v1beta1::BaseAccount, base::v1beta1::Coin},
    cosmwasm::{self, wasm::v1::QuerySmartContractStateRequest},
};
use cosmos_sdk_proto::{traits::Message, Any};
use cosmrs::{
    tendermint::{block::Height, chain},
    tx::{Msg, SignDoc},
};

use cosmrs::{
    cosmwasm::MsgExecuteContract,
    rpc::Client,
    tx::{self, Fee, SignerInfo},
    AccountId,
};
use cw_mantis_order::{Cow, OrderItem, OrderSubMsg, SolutionSubMsg};
use mantis_node::{
    mantis::{
        args::*,
        cosmos::{
            client::*,
            cosmwasm::{smart_query, to_exec_signed, to_exec_signed_with_fund},
            *,
        },
    },
    prelude::*,
    solver::{orderbook::OrderList, solution::Solution},
};

#[tokio::main]
async fn main() {
    let args = MantisArgs::parsed();
    println!("args: {:?}", args);
    let mut wasm_read_client = create_wasm_query_client(&args.grpc_centauri).await;

    let signer = mantis_node::mantis::cosmos::signer::from_mnemonic(
        args.wallet.as_str(),
        "m/44'/118'/0'/0/0",
    )
    .expect("mnemonic");

    let mut cosmos_query_client = create_cosmos_query_client(&args.rpc_centauri).await;
    print!("client 1");
    let mut write_client = create_wasm_write_client(&args.rpc_centauri).await;
    print!("client 2");

    loop {
        if let Some(assets) = args.simulate.clone() {
            if std::time::Instant::now().elapsed().as_millis() % 1000 == 0 {
                let tip = get_latest_block_and_account_by_key(
                    &args.rpc_centauri,
                    &args.grpc_centauri,
                    &signer,
                )
                .await;
                simulate_order(
                    &mut write_client,
                    &mut cosmos_query_client,
                    args.order_contract.clone(),
                    assets,
                    &signer,
                    &args.rpc_centauri,
                    &tip,
                )
                .await;
            };
        };

        if std::time::Instant::now().elapsed().as_millis() % 10000 == 0 {
            let tip = get_latest_block_and_account_by_key(
                &args.rpc_centauri,
                &args.grpc_centauri,
                &signer,
            )
            .await;
            cleanup(
                &mut write_client,
                &mut cosmos_query_client,
                args.order_contract.clone(),
                &signer,
                &args.rpc_centauri,
                &tip,
            )
            .await;
        };

        let tip =
            get_latest_block_and_account_by_key(&args.rpc_centauri, &args.grpc_centauri, &signer)
                .await;

        solve(
            &mut write_client,
            &mut wasm_read_client,
            &args.order_contract,
            &signer,
            &args.rpc_centauri,
            &tip,
        )
        .await;
    }
}

/// `assets` - is comma separate list. each entry is amount u64 glued with alphanumeric denomination
/// that is splitted into array of CosmWasm coins.
/// one coin is chosen as given,
/// from remaining 2 other coins one is chosen as wanted
/// amount of count is randomized around values
///
/// `write_client`
/// `order_contract` - orders are formed for give and want, and send to orders contract.
/// timeout is also randomized starting from 10 to 100 blocks
///
/// Also calls `timeout` so old orders are cleaned.
async fn simulate_order(
    write_client: &mut CosmWasmWriteClient,
    cosmos_query_client: &mut CosmosQueryClient,
    order_contract: String,
    asset: String,
    signing_key: &cosmrs::crypto::secp256k1::SigningKey,
    rpc: &str,
    tip: &Tip,
) {
    println!("========================= simulate_order =========================");
    let coins: Vec<_> = asset
        .split(',')
        .map(|x| cosmwasm_std::Coin::from_str(x).expect("coin"))
        .collect();

    let coins = if rand::random::<bool>() {
        (coins[0].clone(), coins[1].clone())
    } else {
        (coins[1].clone(), coins[0].clone())
    };

    let auth_info = simulate_and_set_fee(signing_key, &tip.account).await;
    let msg = cw_mantis_order::ExecMsg::Order {
        msg: OrderSubMsg {
            wants: cosmwasm_std::Coin {
                amount: coins.0.amount,
                denom: coins.0.denom.clone(),
            },
            transfer: None,
            timeout: tip.timeout(100),
            min_fill: None,
        },
    };
    println!("msg: {:?}", msg);

    let fund = cosmrs::Coin {
        amount: coins.1.amount.into(),
        denom: cosmrs::Denom::from_str(&coins.1.denom).expect("denom"),
    };

    let msg = to_exec_signed_with_fund(signing_key, order_contract, msg, fund);

    tx_broadcast_single_signed_msg(
        msg.to_any().expect("proto"),
        auth_info,
        rpc,
        signing_key,
        &tip,
    )
    .await;

    // here parse contract result for its response
}

async fn cleanup(
    write_client: &mut CosmWasmWriteClient,
    cosmos_query_client: &mut CosmosQueryClient,
    order_contract: String,
    signing_key: &cosmrs::crypto::secp256k1::SigningKey,
    rpc: &str,
    tip: &Tip,
) {
    println!("========================= cleanup =========================");
    let auth_info = simulate_and_set_fee(signing_key, &tip.account).await;
    let msg = cw_mantis_order::ExecMsg::Timeout {
        orders: vec![],
        solutions: vec![],
    };
    let msg = to_exec_signed(signing_key, order_contract, msg);
    tx_broadcast_single_signed_msg(
        msg.to_any().expect("proto"),
        auth_info,
        rpc,
        signing_key,
        tip,
    )
    .await;
}

/// gets orders, groups by pairs
/// solves them using algorithm
/// if any volume solved, posts solution
///
/// gets data from chain pools/fees on osmosis and neutron
/// gets CVM routing data
/// uses cfmm algorithm
async fn solve(
    write_client: &mut CosmWasmWriteClient,
    cosmos_query_client: &mut CosmWasmReadClient,
    order_contract: &String,
    signing_key: &cosmrs::crypto::secp256k1::SigningKey,
    rpc: &str,
    tip: &Tip,
) {
    println!("========================= solve =========================");
    let query = cw_mantis_order::QueryMsg::GetAllOrders {};
    let all_orders: Vec<OrderItem> = smart_query(order_contract, query, cosmos_query_client).await;
    println!("all_orders: {:?}", all_orders);
    if !all_orders.is_empty() {
        let all_orders = all_orders.into_iter().group_by(|x| {
            let mut ab = (x.given.denom.clone(), x.msg.wants.denom.clone());
            ab.sort_selection();
            ab
        });
        for ((a, b), orders) in all_orders.into_iter() {
            let orders = orders.collect::<Vec<_>>();
            use mantis_node::solver::solver::*;
            use mantis_node::solver::types::*;
            let orders = orders.iter().map(|x| {
                let (side, price) = if x.given.denom == a {
                    (
                        OrderType::Buy,
                        Price::new_float(
                            x.msg.wants.amount.u128() as f64 / x.given.amount.u128() as f64,
                        ),
                    )
                } else {
                    (
                        OrderType::Sell,
                        Price::new_float(
                            x.given.amount.u128() as f64 / x.msg.wants.amount.u128() as f64,
                        ),
                    )
                };

                mantis_node::solver::types::Order::new(
                    Amount::from_f64_retain(x.given.amount.u128() as f64).expect("decimal"),
                    price,
                    side,
                    x.order_id,
                )
            });
            let orders = OrderList {
                value: orders.collect(),
            };
            orders.print();
            let optimal_price = orders.compute_optimal_price(1000);
            let mut solution = Solution::new(orders.value.clone());
            solution = solution.match_orders(optimal_price);
            solution.print();
            let cows = solution
                .orders
                .value
                .into_iter()
                .filter(|x| x.amount_filled > <_>::default())
                .map(|x| {
                    let filled = x.amount_filled.ceil().to_u128().expect("u128");
                    Cow {
                        order_id: x.id,
                        cow_amount: filled.into(),
                        given: filled.into(),
                    }
                });
            let solution = SolutionSubMsg {
                cows: cows.collect(),
                route: None,
                timeout: tip.timeout(12),
            };

            let msg = cw_mantis_order::ExecMsg::Solve { msg: solution };
        }
    }
}
