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
use cw_mantis_order::{Amount, OrderItem, OrderSolution, OrderSubMsg, SolutionSubMsg};
use mantis_node::{
    mantis::{
        args::*,
        cosmos::{
            client::*,
            cosmwasm::{smart_query, to_exec_signed, to_exec_signed_with_fund},
            *,
        },
        mantis::randomize_order,
    },
    prelude::*,
    solver::{orderbook::OrderList, solution::Solution},
};

#[tokio::main]
async fn main() {
    let args = MantisArgs::parsed();
    match args.command {
        MantisCommands::Solve(args) => {
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

                if std::time::Instant::now().elapsed().as_millis() % 100000 == 0 {
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

                let tip = get_latest_block_and_account_by_key(
                    &args.rpc_centauri,
                    &args.grpc_centauri,
                    &signer,
                )
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
        MantisCommands::Id(args) => match args.command {
            IdCommands::Asset(args) => match args.command {
                AssetCommands::Gen {
                    network_id,
                    asset_id,
                } => {
                    println!(
                        "{}",
                        cvm_runtime::generate_asset_id(network_id.into(), 0, asset_id.into())
                    );
                }
            },
        },
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
    coins_pair: String,
    signing_key: &cosmrs::crypto::secp256k1::SigningKey,
    rpc: &str,
    tip: &Tip,
) {
    println!("========================= simulate_order =========================");
    let (msg, fund) = randomize_order(coins_pair, tip.block);

    println!("msg: {:?}", msg);

    let auth_info = simulate_and_set_fee(signing_key, &tip.account).await;

    let msg = to_exec_signed_with_fund(signing_key, order_contract, msg, fund);

    let result = tx_broadcast_single_signed_msg(
        msg.to_any().expect("proto"),
        auth_info,
        rpc,
        signing_key,
        &tip,
    )
    .await;

    println!("simulated tx {:?}", result.height)
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
    let all_orders = get_all_orders(order_contract, cosmos_query_client, tip).await;
    if !all_orders.is_empty() {
        let cows_per_pair = mantis_node::mantis::mantis::do_cows(all_orders);
        for (cows, optimal_price) in cows_per_pair {
            send_solution(cows, tip, optimal_price, signing_key, order_contract, rpc).await;
        }
    }
}

async fn send_solution(
    cows: Vec<OrderSolution>,
    tip: &Tip,
    optimal_price: (u64, u64),
    signing_key: &cosmrs::crypto::secp256k1::SigningKey,
    order_contract: &String,
    rpc: &str,
) {
    println!("========================= settle =========================");
    let solution = SolutionSubMsg {
        cows,
        route: None,
        timeout: tip.timeout(12),
        cow_optional_price: optimal_price.into(),
    };

    let auth_info = simulate_and_set_fee(signing_key, &tip.account).await;
    let msg = cw_mantis_order::ExecMsg::Solve { msg: solution };
    let msg = to_exec_signed(signing_key, order_contract.clone(), msg);
    let result = tx_broadcast_single_signed_msg(
        msg.to_any().expect("proto"),
        auth_info,
        rpc,
        signing_key,
        tip,
    )
    .await;
    println!("result: {:?}", result);
}

async fn get_all_orders(
    order_contract: &String,
    cosmos_query_client: &mut CosmWasmReadClient,
    tip: &Tip,
) -> Vec<OrderItem> {
    let query = cw_mantis_order::QueryMsg::GetAllOrders {};
    let all_orders = smart_query::<_, Vec<OrderItem>>(order_contract, query, cosmos_query_client)
        .await
        .into_iter()
        .filter(|x| x.msg.timeout > tip.block.value())
        .collect::<Vec<OrderItem>>();
    println!("all_orders: {:?}", all_orders);
    all_orders
}
