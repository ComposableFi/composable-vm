use blackbox_rs::{types::*, Client};
use bounded_collections::Get;
/// Given total amount it, order owners and desired out, produce CVM program from and by requesting route
use cvm_runtime::{
    outpost::GetConfigResponse,
    shared::{CvmFundsFilter, CvmInstruction, CvmProgram},
    Amount,
};
use cw_mantis_order::{CrossChainPart, OrderItem, SolutionSubMsg};

use crate::solver::router::shortest_path;

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
            NextItem::Exchange(exchange) => {
                let exchange = new_exchange(exchange);
                current.instructions.push(exchange);
                build_next(current, rest, glt, salt);
            }
            NextItem::Spawn(spawn) => {
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
    spawn: &Spawn,
    program: CvmProgram,
    glt: &GetConfigResponse,
    salt: &[u8],
) -> CvmInstruction {
    let in_asset_id = match spawn.in_asset_id.as_ref().expect("in_asset_id") {
        InAssetId::Variant1(id) => id.parse().expect("in_asset_id"),
        _ => panic!("in_asset_id"),
    };

    let in_amount: Amount = match spawn.in_asset_amount.as_ref().expect("in_asset_amount") {
        InAssetAmount::Variant0(x) => (*x).try_into().expect("in_asset_amount"),
        InAssetAmount::Variant1(x) => x.parse().expect("in_asset_amount"),
        InAssetAmount::Variant2(x) => {
            panic!("fix python not to have float or use to fixed point/fraction first")
        } // Amount::try_floor_f64(*x).expect("in_asset_amount"),
    };

    let out_asset_id = match &spawn.out_asset_id {
        OutAssetId::Variant1(id) => id.parse().expect("in_asset_id"),
        _ => panic!("in_asset_id"),
    };

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

fn new_exchange(exchange: &Exchange) -> CvmInstruction {
    let exchange_id = match &exchange.pool_id {
        PoolId::Variant1(id) => id.parse().expect("pool id"),
        _ => panic!("exchange_id"),
    };
    let in_asset_id = match &exchange.in_asset_id {
        InAssetId::Variant1(id) => id.parse().expect("in_asset_id"),
        _ => panic!("in_asset_id"),
    };

    let in_amount: Amount = match &exchange.in_asset_amount {
        InAssetAmount::Variant0(x) => (*x).try_into().expect("in_asset_amount"),
        InAssetAmount::Variant1(x) => x.parse().expect("in_asset_amount"),
        InAssetAmount::Variant2(x) => {
            panic!("covert f64 to fraction, but really just fix python to give strings")
        } // Amount::try_floor_f64(*x).expect("in_asset_amount"),
    };

    let out_asset_id = match &exchange.out_asset_id {
        OutAssetId::Variant1(id) => id.parse().expect("in_asset_id"),
        _ => panic!("in_asset_id"),
    };

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
) -> Vec<CvmInstruction> {
    if route_provider == "shortest_path" {
        return shortest_path::route(cvm_glt, input, salt);
    } else {
        let blackbox: Client = Client::new(route_provider);
        let mut route = blackbox
            .simulator_router_simulator_router_get(
                &InAssetAmount::Variant0(
                    input.in_asset_amount.0.try_into().expect("in_asset_amount"),
                ),
                &InAssetId::Variant1(input.in_asset_id.to_string()),
                true,
                &OutAssetAmount::Variant0(10),
                &OutAssetId::Variant1(input.out_asset_id.to_string().into()),
            )
            .await
            .expect("route found")
            .into_inner()
            .pop()
            .expect("at least one route");

        let mut program = CvmProgram::default();
        build_next(&mut program, &mut route.next, cvm_glt, salt);
        panic!("so need to build instruction so can plug into one program (transaciton)")
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
        let cvm_program = if let Some(ref cvm_glt) = cvm_glt {
            let cvm_program = intent_banks_to_cvm_program(
                pair_solution.clone(),
                &active_orders,
                cvm_glt,
                router,
                &salt,
            )
            .await;

            Some(cvm_program)
        } else {
            None
        };

        // would be reasonable to do do cross chain if it solves some % of whole trade
        let route = if let Some(cvm_program) = cvm_program
            && Decider::get()
        {
            Some(CrossChainPart::new(
                cvm_program,
                salt.clone(),
                pair_solution.optimal_price.into(),
            ))
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
) -> CvmProgram {
    let intents = IntentBankInput::find_intent_amount(
        pair_solution.cows.as_ref(),
        all_orders,
        pair_solution.optimal_price,
        cvm_glt,
        pair_solution.ab.clone(),
    );

    log::info!(target:"mantis::solver::", "intents: {:?}", intents);

    let mut instructions = vec![];

    for intent in intents.into_iter().filter(|x| x.in_asset_amount.0.gt(&0)) {
        let mut cvm_routes = get_routes(router_api, intent, cvm_glt, salt.as_ref()).await;
        instructions.append(&mut cvm_routes);
    }

    log::info!(target: "mantis::solver", "built instructions: {:?}", instructions);

    let cvm_program = CvmProgram {
        tag: salt.to_vec(),
        instructions,
    };
    cvm_program
}
