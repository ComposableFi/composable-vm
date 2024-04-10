#![feature(let_chains)]

use std::panic;

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
        MantisCommands::Glt(_) => todo!(),
    }
}

async fn solve_orders(solver_args: &SolverArgs) {
    let args = &solver_args.shared;
    let _wasm_read_client = create_wasm_query_client(&args.grpc_centauri).await;

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
        let stale_orders =
            mantis_node::mantis::indexer::get_stale_orders(&args.order_contract, &mut wasm_read_client, &tip).await;
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
            solve(
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

    let pair = simulate_args
        .coins
        .choose(&mut rand::thread_rng())
        .expect("some");
    let rpc = CosmosChainInfo {
        rpc: args.rpc_centauri.clone(),
        chain_id: args.main_chain_id.clone(),
    };
    simulate::simulate_order(
        &mut write_client,
        &mut cosmos_query_client,
        args.order_contract.clone(),
        pair,
        &signer,
        &rpc,
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
    _write_client: &mut CosmWasmWriteClient,
    cosmos_query_client: &mut CosmWasmReadClient,
    // really this should query Python Blackbox
    cvm_contact: &Option<String>,
    order_contract: &String,
    signing_key: &cosmrs::crypto::secp256k1::SigningKey,
    rpc: &CosmosChainInfo,
    tip: &Tip,
    gas: Gas,
    all_orders: Vec<OrderItem>,
    router_api: &String,
) {
    log::info!(target: "mantis::solver", "Solving orders");

    let cows_per_pair = mantis_node::mantis::solve::find_cows(&all_orders);
    for pair_solution in cows_per_pair {
        let salt = crate::cvm::get_salt(signing_key, tip);
        let cvm_program = if let Some(cvm_contact) = cvm_contact {
            let cvm_glt = get_cvm_glt(cvm_contact, cosmos_query_client).await;
            let (a, b) = mantis_node::mantis::solve::IntentBankInput::find_intent_amount(
                pair_solution.cows.as_ref(),
                &all_orders,
                pair_solution.optimal_price,
                &cvm_glt,
                pair_solution.ab.clone(),
            );

            let a_cvm_route = blackbox::get_route(router_api, a, &cvm_glt, salt.as_ref()).await;
            let b_cvm_route = blackbox::get_route(router_api, b, &cvm_glt, salt.as_ref()).await;

            Some(CvmProgram {
                tag: salt.to_vec(),
                instructions: [a_cvm_route, b_cvm_route].concat().to_vec(),
            })
        } else {
            None
        };
        send_solution(
            pair_solution.cows,
            cvm_program,
            tip,
            pair_solution.optimal_price.into(),
            signing_key,
            order_contract,
            rpc,
            gas,
            salt.to_vec(),
        )
        .await;
    }
}

async fn send_solution(
    cows: Vec<OrderSolution>,
    program: Option<CvmProgram>,
    tip: &Tip,
    optimal_price: Ratio,
    signing_key: &cosmrs::crypto::secp256k1::SigningKey,
    order_contract: &String,
    rpc: &CosmosChainInfo,
    gas: Gas,
    salt: Vec<u8>,
) {
    log::info!("========================= settle =========================");
    // would be reasonable to do do cross chain if it solves some % of whole trade
    let route = if let Some(program) = program
        && random::<bool>()
    {
        Some(CrossChainPart {
            msg: cvm_runtime::outpost::ExecuteProgramMsg {
                salt,
                program,
                assets: None,
                tip: None,
            },
            optimal_price,
        })
    } else {
        None
    };
    let solution = SolutionSubMsg {
        cows,
        route,
        timeout: tip.timeout(12),
        optimal_price,
    };

    let auth_info = simulate_and_set_fee(signing_key, &tip.account, gas).await;
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
