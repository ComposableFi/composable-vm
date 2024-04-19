use blackbox_rs::{types::*, Client};
use bounded_collections::Get;
/// Given total amount it, order owners and desired out, produce CVM program from and by requesting route
use cvm_runtime::{
    outpost::GetConfigResponse,
    shared::{CvmFundsFilter, CvmInstruction, CvmProgram},
    Amount, AssetId, ExchangeId,
};
use cw_cvm_outpost::msg::CvmGlt;
use cw_mantis_order::{CrossChainPart, OrderItem, SolutionSubMsg};

use crate::{error::MantisError, solver::router::shortest_path};

use super::{
    cosmos::client::Tip,
    solve::{find_cows, IntentBankInput, PairSolution},
};

/// given route and CVM stub with amount, build it to the end
fn build_next(
    current: &mut CvmProgram,
    next: &mut [NextItem],
    glt: &GetConfigResponse,
    salt: &[u8],
) {
    match next.split_first_mut() {
        Some((head, rest)) => match head {
            NextItem::ExchangeStrStr(exchange) => {
                let exchange = new_exchange(exchange);
                current.instructions.push(exchange);
                build_next(current, rest, glt, salt);
            }
            NextItem::SpawnStrStr(spawn) => {
                let mut program = CvmProgram::default();
                build_next(&mut program, rest, glt, salt);
                let spawn = new_spawn(spawn, program, glt, salt);
                current.instructions.push(spawn);
            }
        },
        None => {
            log::info!("no more routes");
        }
    }
}

fn new_spawn(
    spawn: &SpawnStrStr,
    program: CvmProgram,
    glt: &GetConfigResponse,
    salt: &[u8],
) -> CvmInstruction {
    let in_asset_id: AssetId = spawn.in_asset_id.parse().expect("in_asset_id");

    let in_amount: Amount = spawn.in_asset_amount.parse().expect("in_asset_amount");
    let out_asset_id = spawn.out_asset_id.parse().expect("out_asset_id");

    let network_id = glt
        .assets
        .iter()
        .find(|x| x.asset_id == out_asset_id)
        .map(|x| x.network_id)
        .expect("network_id");
    CvmInstruction::Spawn {
        program,
        network_id,
        salt: salt.to_vec(),
        assets: CvmFundsFilter::of(in_asset_id, in_amount),
    }
}

fn new_exchange(exchange: &ExchangeStrStr) -> CvmInstruction {
    let exchange_id: ExchangeId = exchange.pool_id.parse().expect("pool_id");
    let in_asset_id: AssetId = exchange.in_asset_id.parse().expect("in_asset_id");

    let in_amount: Amount = exchange.in_asset_amount.parse().expect("in_asset_amount");

    let out_asset_id: AssetId = exchange.out_asset_id.parse().expect("in_asset_id");

    CvmInstruction::Exchange {
        exchange_id,
        give: CvmFundsFilter::of(in_asset_id, in_amount),
        want: CvmFundsFilter::one_of(out_asset_id),
    }
}

pub async fn get_routes(
    route_provider: &str,
    input: IntentBankInput,
    cvm_glt: &GetConfigResponse,
    salt: &[u8],
) -> Result<Vec<CvmInstruction>, MantisError> {
    if route_provider == "shortest_path" {
        Ok(shortest_path::route(cvm_glt, input, salt))
    } else {
        let blackbox: Client = Client::new(route_provider);
        let mut route = blackbox
            .simulator_router_simulator_router_get(
                input.in_asset_amount.to_string().as_str().into(),
                input.in_asset_id.to_string().as_str().into(),
                Some(true),
                10.to_string().as_str().into(),
                input.out_asset_id.to_string().as_str().into(),
            )
            .await
            .map_err(|x| MantisError::BlackboxError {
                reason: x.to_string(),
            })?
            .into_inner()
            .pop()
            .ok_or(MantisError::BlackboxError {
                reason: "no routes".to_string(),
            })?;

        log::info!("route: {:?}", route);
        Ok(build_instructions(
            input.order_accounts,
            &route.next[0],
            cvm_glt,
            salt,
        ))
    }
}

fn build_instructions(
    mut final_instructions: Vec<CvmInstruction>,
    route: &NextItem,
    cvm_glt: &CvmGlt,
    salt: &[u8],
) -> Vec<CvmInstruction> {
    match route {
        NextItem::ExchangeStrStr(exchange) => {
            let ix = CvmInstruction::Exchange {
                exchange_id: exchange.pool_id.parse().expect("pool_id"),
                give: CvmFundsFilter::all_of(exchange.in_asset_id.parse().expect("in")),
                want: CvmFundsFilter::one_of(exchange.out_asset_id.parse().expect("out")),
            };

            let mut ixs = vec![];
            if let Some(next) = exchange.next.get(0) {
                let mut next = build_instructions(final_instructions, next, cvm_glt, salt);
                ixs.push(ix);
                ixs.append(&mut next);
                ixs
            } else {
                ixs.append(&mut final_instructions);
                ixs
            }
        }
        NextItem::SpawnStrStr(spawn) => {
            if let Some(next) = spawn.next.get(0) {
                let mut next = build_instructions(final_instructions, next, cvm_glt, salt);
                let program = CvmProgram {
                    tag: salt.to_vec(),
                    instructions: next,
                };
                let to_asset_id = spawn.out_asset_id.parse().expect("out");
                let spawn = CvmInstruction::Spawn {
                    network_id: cvm_glt.get_network_for_asset(to_asset_id),
                    salt: salt.to_vec(),
                    assets: CvmFundsFilter::all_of(spawn.in_asset_id.parse().expect("in")),
                    program,
                };
                vec![spawn]
            } else {
                let program = CvmProgram {
                    tag: salt.to_vec(),
                    instructions: final_instructions,
                };
                let to_asset_id = spawn.out_asset_id.parse().expect("out");
                let spawn = CvmInstruction::Spawn {
                    network_id: cvm_glt.get_network_for_asset(to_asset_id),
                    salt: salt.to_vec(),
                    assets: CvmFundsFilter::all_of(spawn.in_asset_id.parse().expect("in")),
                    program,
                };
                vec![spawn]
            }
        }
    }
}

pub async fn solve<Decider: Get<bool>>(
    active_orders: Vec<OrderItem>,
    signing_key: &cosmrs::crypto::secp256k1::SigningKey,
    tip: &Tip,
    cvm_glt: Option<cw_cvm_outpost::msg::CvmGlt>,
    router: &str,
) -> Vec<cw_mantis_order::ExecMsg> {
    let cows_per_pair = find_cows(&active_orders);
    let mut msgs = vec![];

    for pair_solution in cows_per_pair {
        let salt = super::cosmos::cvm::calculate_salt(signing_key, tip, pair_solution.ab.clone());

        // would be reasonable to do do cross chain if it solves some % of whole trade
        let route = if Decider::get() {
            let cvm_program = if let Some(ref cvm_glt) = cvm_glt {
                let cvm_program = intent_banks_to_cvm_program(
                    pair_solution.clone(),
                    &active_orders,
                    cvm_glt,
                    router,
                    &salt,
                )
                .await;

                if cvm_program.is_err() {
                    log::error!("cvm_program error: {:?}", cvm_program);
                }
                cvm_program.ok()
            } else {
                None
            };

            cvm_program.map(|cvm_program| {
                CrossChainPart::new(
                    cvm_program,
                    salt.clone(),
                    pair_solution.optimal_price.into(),
                )
            })
        } else {
            None
        };
        let msg = SolutionSubMsg {
            cows: pair_solution.cows.clone(),
            route,
            timeout: tip.timeout(12),
            optimal_price: pair_solution.optimal_price.into(),
        };
        let msg = cw_mantis_order::ExecMsg::Solve { msg };
        msgs.push(msg);
    }
    msgs
}

async fn intent_banks_to_cvm_program(
    pair_solution: PairSolution,
    all_orders: &Vec<OrderItem>,
    cvm_glt: &cw_cvm_outpost::msg::GetConfigResponse,
    router_api: &str,
    salt: &Vec<u8>,
) -> Result<CvmProgram, MantisError> {
    let intents = IntentBankInput::find_intent_amount(
        pair_solution.cows.as_ref(),
        all_orders,
        pair_solution.optimal_price,
        cvm_glt,
        pair_solution.ab.clone(),
    )?;

    log::info!(target:"mantis::solver::", "intents: {:?}", intents);

    let mut instructions = vec![];

    for intent in intents.into_iter().filter(|x| x.in_asset_amount.0.gt(&0)) {
        let mut cvm_routes = get_routes(router_api, intent, cvm_glt, salt.as_ref()).await?;
        instructions.append(&mut cvm_routes);
    }

    log::info!(target: "mantis::solver", "built instructions: {:?}", instructions);

    let cvm_program = CvmProgram {
        tag: salt.to_vec(),
        instructions,
    };
    Ok(cvm_program)
}
