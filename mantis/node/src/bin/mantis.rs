use bip32::secp256k1::elliptic_curve::rand_core::block;
use cosmos_sdk_proto::{
    cosmos::{auth::v1beta1::BaseAccount, base::v1beta1::Coin},
    cosmwasm::{self, wasm::v1::QuerySmartContractStateRequest},
};
use cosmos_sdk_proto::{traits::Message, Any};
use cosmrs::{
    tendermint::{block::Height, chain},
    tx::{Msg, SignDoc},
    Gas,
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
        args::*, autopilot, blackbox, cosmos::{
            client::*,
            cosmwasm::{smart_query, to_exec_signed, to_exec_signed_with_fund},
            *,
        }, indexer::{get_all_orders, get_cvm_glt}, simulate
    },
    prelude::*,
    solver::{orderbook::OrderList, solution::Solution},
};

#[tokio::main]
async fn main() {
    let args = MantisArgs::parsed();

    match &args.command {
        MantisCommands::Solve(_) => todo!(),
        MantisCommands::Simulate(x) => {
            simulate_orders(x).await;
        }
        
        MantisCommands::Id(_) => todo!(),
        MantisCommands::Glt(_) => todo!(),
    }

    // match args.command {
    //     MantisCommands::Solve(args) => {
    //         let mut wasm_read_client = create_wasm_query_client(&args.grpc_centauri).await;

    //         let signer = mantis_node::mantis::cosmos::signer::from_mnemonic(
    //             args.wallet.as_str(),
    //             "m/44'/118'/0'/0/0",
    //         )
    //         .expect("mnemonic");
    //         let gas = args.gas;
    //         let mut cosmos_query_client = create_cosmos_query_client(&args.rpc_centauri).await;
    //         let mut write_client = create_wasm_write_client(&args.rpc_centauri).await;
            

    //         let start = std::time::Instant::now();
    //         loop {

    //             if start.elapsed().as_millis() % 10000 == 0 {
    //                 let tip = get_latest_block_and_account_by_key(
    //                     &args.rpc_centauri,
    //                     &args.grpc_centauri,
    //                     &signer,
    //                 )
    //                 .await;
    //                 autopilot::cleanup(
    //                     &mut write_client,
    //                     &mut cosmos_query_client,
    //                     args.order_contract.clone(),
    //                     &signer,
    //                     &args.rpc_centauri,
    //                     &tip,
    //                     gas,
    //                 )
    //                 .await;
    //             };

    //             let tip = get_latest_block_and_account_by_key(
    //                 &args.rpc_centauri,
    //                 &args.grpc_centauri,
    //                 &signer,
    //             )
    //             .await;

    //             solve(
    //                 &mut write_client,
    //                 &mut wasm_read_client,
    //                 &args.order_contract,
    //                 &signer,
    //                 &args.rpc_centauri,
    //                 &tip,
    //                 gas,
    //             )
    //             .await;
    //         }
    //     }
    //     MantisCommands::Id(args) => match args.command {
    //         IdCommands::Asset(args) => match args.command {
    //             AssetCommands::Gen {
    //                 network_id,
    //                 asset_id,
    //             } => {
    //                 println!(
    //                     "{}",
    //                     cvm_runtime::generate_asset_id(network_id.into(), 0, asset_id.into())
    //                 );
    //             }
    //         },
    //     },
    // }

}

async fn simulate_orders(simulate_args: &SimulateArgs)  {
    let args = simulate_args.shared;
    let mut wasm_read_client = create_wasm_query_client(&args.).await;

            let signer = mantis_node::mantis::cosmos::signer::from_mnemonic(
                args.wallet.as_str(),
                "m/44'/118'/0'/0/0",
            )
            .expect("mnemonic");
            let gas = args.gas;
            let mut cosmos_query_client = create_cosmos_query_client(&args.rpc_centauri).await;
            let mut write_client = create_wasm_write_client(&args.rpc_centauri).await;
    log::info!("Simulating orders");

    let tip = get_latest_block_and_account_by_key(
        &args.rpc_centauri,
        &args.grpc_centauri,
        &signer,
    )
    .await;
    simulate::simulate_order(
        &mut write_client,
        &mut cosmos_query_client,
        args.order_contract.clone(),
        assets,
        &signer,
        &args.rpc_centauri,
        &tip,
        gas,
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
    // really this should query Python Blackbox
    cvm_contact: &String,
    order_contract: &String,
    signing_key: &cosmrs::crypto::secp256k1::SigningKey,
    rpc: &str,
    tip: &Tip,
    gas: Gas,
) {

    let salt = get_salt(signing_key, tip);
    println!("========================= solve =========================");
    let all_orders = get_all_orders(order_contract, cosmos_query_client, tip).await;
    if !all_orders.is_empty() {
        let cows_per_pair = mantis_node::mantis::solve::do_cows(all_orders);
        let cvm_glt = get_cvm_glt(cvm_contact, &mut cosmos_query_client).await;
        let cows_cvm = blackbox::route(cows_per_pair, all_orders, &cvm_glt, salt.as_ref()).await;
        for (cows, optimal_price) in cows_per_pair {
            send_solution(
                cows,
                tip,
                optimal_price,
                signing_key,
                order_contract,
                rpc,
                gas,
            )
            .await;
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
    gas: Gas,
) {
    println!("========================= settle =========================");
    let solution = SolutionSubMsg {
        cows,
        route: None,
        timeout: tip.timeout(12),
        cow_optional_price: optimal_price.into(),
    };

    let auth_info = simulate_and_set_fee(signing_key, &tip.account, gas).await;
    let msg = cw_mantis_order::ExecMsg::Settle { msg: solution };
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

