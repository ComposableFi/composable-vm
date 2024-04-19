#![feature(let_chains)]

use std::panic;

use bounded_collections::Get;
use cosmrs::{tx::Msg, Gas};

use cvm_runtime::shared::CvmProgram;
use cw_mantis_order::{CrossChainPart, OrderItem, OrderSolution, Ratio, SolutionSubMsg};
use mantis_node::{
    mantis::{
        args::*,
        autopilot, blackbox,
        cosmos::{client::*, cosmwasm::to_exec_signed, *},
        indexer::{get_active_orders, get_cvm_glt},
        simulate,
        solve::PairSolution,
    },
    prelude::*,
};

#[tokio::main]
async fn main() {
    let args = MantisArgs::parsed();
    env_logger::init();

    match &args.command {
        MantisCommands::Solve(x) => solve_orders(x).await,
        MantisCommands::Simulate(x) => {
            simulate_orders(x).await;
        }

        MantisCommands::Id(x) => match &x.command {
            IdCommands::Asset(args) => match args.command {
                AssetCommands::Gen {
                    network_id,
                    asset_id,
                } => {
                    println!(
                        "{}",
                        cvm_runtime::generate_asset_id(network_id.into(), 0, asset_id)
                    );
                }
            },
        },
        MantisCommands::Glt(x) => match &x.command {
            GltCommands::Validate => todo!(),
            GltCommands::Plan => todo!(),
            GltCommands::Add => todo!(),
            GltCommands::Get(args) => {
                let mut wasm_read_client = create_wasm_query_client(&args.grpc_centauri).await;
                let cvm_glt = get_cvm_glt(&args.cvm_contract, &mut wasm_read_client).await;
                println!("{:?}", cvm_glt);
            }
        },
    }
}

async fn solve_orders(solver_args: &SolverArgs) {
    let args = &solver_args.shared;

    let signer = mantis_node::mantis::cosmos::signer::from_mnemonic(
        args.wallet.as_str(),
        "m/44'/118'/0'/0/0",
    )
    .expect("mnemonic");
    let _gas = args.gas;
    let mut cosmos_query_client = create_cosmos_query_client(&args.rpc_centauri).await;
    let mut write_client = create_wasm_write_client(&args.rpc_centauri).await;
    let mut wasm_read_client = create_wasm_query_client(&args.grpc_centauri).await;
    let gas = args.gas;

    let cosmos_chain_info = CosmosChainInfo {
        rpc: args.rpc_centauri.clone(),
        chain_id: args.main_chain_id.clone(),
    };
    loop {
        let tip =
            get_latest_block_and_account_by_key(&args.rpc_centauri, &args.grpc_centauri, &signer)
                .await;
        let stale_orders = mantis_node::mantis::indexer::get_stale_orders(
            &args.order_contract,
            &mut wasm_read_client,
            &tip,
        )
        .await;
        if stale_orders.len() > 0 {
            log::warn!(target: "mantis::autopilot", "timedouted orders");
            autopilot::cleanup(
                &mut write_client,
                &mut cosmos_query_client,
                args.order_contract.clone(),
                &signer,
                &cosmos_chain_info,
                &tip,
                gas,
            )
            .await;
        }

        let tip =
            get_latest_block_and_account_by_key(&args.rpc_centauri, &args.grpc_centauri, &signer)
                .await;
        let all_orders = get_active_orders(&args.order_contract, &mut wasm_read_client, &tip).await;
        if !all_orders.is_empty() {
            let main_chain = CosmosChainInfo {
                rpc: args.rpc_centauri.clone(),
                chain_id: args.main_chain_id.clone(),
            };
            get_data_and_solve(
                &mut write_client,
                &mut wasm_read_client,
                &solver_args.cvm_contract,
                &args.order_contract,
                &signer,
                &main_chain,
                &tip,
                gas,
                all_orders,
                &solver_args.router,
            )
            .await;
        };
    }
}

async fn simulate_orders(simulate_args: &SimulateArgs) {
    let args = &simulate_args.shared;
    let _wasm_read_client = create_wasm_query_client(&args.grpc_centauri).await;

    let signer = mantis_node::mantis::cosmos::signer::from_mnemonic(
        args.wallet.as_str(),
        "m/44'/118'/0'/0/0",
    )
    .expect("mnemonic");
    let gas = args.gas;
    let mut cosmos_query_client = create_cosmos_query_client(&args.rpc_centauri).await;
    let mut write_client = create_wasm_write_client(&args.rpc_centauri).await;
    log::info!("Simulating orders");

    let tip =
        get_latest_block_and_account_by_key(&args.rpc_centauri, &args.grpc_centauri, &signer).await;

    assert!(simulate_args.coins.len() > 0);
    let mut coinpairs = simulate_args.coins.clone();
    coinpairs.shuffle(&mut rand::thread_rng());

    for coin_pair in coinpairs {
        let rpc = CosmosChainInfo {
            rpc: args.rpc_centauri.clone(),
            chain_id: args.main_chain_id.clone(),
        };
        simulate::simulate_order(
            &mut write_client,
            &mut cosmos_query_client,
            args.order_contract.clone(),
            &coin_pair,
            &signer,
            &rpc,
            &tip,
            gas,
            simulate_args.random_parts,
        )
        .await;
    }
}

enum CoinToss {}

impl Get<bool> for CoinToss {
    fn get() -> bool {
        random::<bool>()
    }
}

/// gets orders, groups by pairs
/// solves them using algorithm
/// if any volume solved, posts solution
///
/// gets data from chain pools/fees on osmosis and neutron
/// gets CVM routing data
/// uses cfmm algorithm
async fn get_data_and_solve(
    _write_client: &mut CosmWasmWriteClient,
    cosmos_query_client: &mut CosmWasmReadClient,
    // really this should query Python Blackbox
    cvm_contact: &Option<String>,
    order_contract: &String,
    signing_key: &cosmrs::crypto::secp256k1::SigningKey,
    rpc: &CosmosChainInfo,
    tip: &Tip,
    gas: Gas,
    all_active_orders: Vec<OrderItem>,
    router_api: &String,
) {
    log::info!(target: "mantis::solver", "Solving orders");
    let cvm_glt = match cvm_contact {
        Some(x) => Some(get_cvm_glt(&x, cosmos_query_client).await),
        None => None,
    };

    let msgs = mantis_node::mantis::blackbox::solve::<CoinToss>(
        all_active_orders,
        signing_key,
        tip,
        cvm_glt,
        router_api,
    )
    .await;

    for msg in msgs {
        send_solution(msg, tip, signing_key, order_contract, rpc, gas).await;
    }
}

async fn send_solution(
    msg: cw_mantis_order::ExecMsg,
    tip: &Tip,
    signing_key: &cosmrs::crypto::secp256k1::SigningKey,
    order_contract: &String,
    rpc: &CosmosChainInfo,
    gas: Gas,
) {
    log::info!("========================= settle =========================");
    let auth_info = simulate_and_set_fee(signing_key, &tip.account, gas).await;
    let msg = to_exec_signed(signing_key, order_contract.clone(), msg);
    let result = tx_broadcast_single_signed_msg(
        msg.to_any().expect("proto"),
        auth_info,
        rpc,
        signing_key,
        tip,
    )
    .await;
    match &result.tx_result.code {
        cosmrs::tendermint::abci::Code::Err(err) => {
            log::info!("result: {:?}", result);
            panic!("Error: {:?}", err)
        }
        cosmrs::tendermint::abci::Code::Ok => {
            log::info!("ok: {:?}", result);
        }
    }
}
