use blackbox_rs::{prelude::*, types::*, Client};
/// Given total amount it, order owners and desired out, produce CVM program from and by requesting route
use cvm_runtime::{
    network,
    outpost::GetConfigResponse,
    proto::cvm,
    shared::{CvmFundsFilter, CvmInstruction, CvmProgram, Displayed, XcAddr},
    Amount, AssetId, ExchangeId,
};

/// input batched summarized from users for routing
struct BankInput {
    pub in_asset_id: AssetId,
    pub in_asset_amount: Displayed<u64>,
    pub out_asset_id: AssetId,
    pub order_accounts: Vec<(XcAddr, Amount)>,
}

impl BankInput {
    pub fn new(
        in_asset_id: AssetId,
        in_asset_amount: Displayed<u64>,
        out_asset_id: AssetId,
        order_accounts: Vec<(XcAddr, Amount)>,
    ) -> Self {
        Self {
            in_asset_id,
            in_asset_amount,
            out_asset_id,
            order_accounts,
        }
    }
}

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
                build_next(current, rest, &glt, salt);
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
        InAssetAmount::Variant2(x) => Amount::try_floor_f64(*x).expect("in_asset_amount"),
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
        assets: CvmFundsFilter::one(in_asset_id, in_amount),
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
        InAssetAmount::Variant2(x) => Amount::try_floor_f64(*x).expect("in_asset_amount"),
    };

    let out_asset_id = match &exchange.out_asset_id {
        OutAssetId::Variant1(id) => id.parse().expect("in_asset_id"),
        _ => panic!("in_asset_id"),
    };

    CvmInstruction::Exchange {
        exchange_id,
        give: CvmFundsFilter::one(in_asset_id, in_amount),
        want: CvmFundsFilter::one(out_asset_id, Amount::one()),
    }
}

/// `order_accounts` - account of order where to dispatch amounts (part of whole)
pub async fn route(
    server: &str,
    input: BankInput,
    glt: &GetConfigResponse,
    salt: &[u8],
) -> CvmProgram {
    let blackbox: Client = Client::new(server);
    let mut route = blackbox
        .simulator_router_simulator_router_get(
            &InAssetAmount::Variant0(input.in_asset_amount.0.try_into().expect("in_asset_amount")),
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
    build_next(&mut program, &mut route.next, glt, salt);
    return program;
}
