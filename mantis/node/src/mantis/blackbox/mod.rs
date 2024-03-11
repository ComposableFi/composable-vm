use blackbox_rs::{prelude::*, types::*, Client};
/// Given total amount it, order owners and desired out, produce CVM program from and by requesting route
use cvm_runtime::{
    outpost::GetConfigResponse, proto::cvm, shared::{CvmInstruction, CvmProgram, Displayed, XcAddr}, Amount, AssetId, ExchangeId 
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
fn build_next(current: &mut CvmProgram, next: &mut [NextItem]) -> CvmInstruction {
    match next.split_first_mut() {
        Some((head, rest)) => {
            match head {
                NextItem::Exchange(exchange) => {
                    let exchange = new_exchange(exchange);
                    current.instructions.push(CvmInstruction::Exchange(exchange));
                    build_next(current, rest)
                },
                NextItem::Spawn(spawn) => {
                    let spawn = new_spawn(spawn);
                    current.instructions.push(CvmInstruction::Spawn(spawn));
                    build_next(spawn.program, rest)
                },
            }
        }
        None => info!("no more routes"),
    }
}

/// `order_accounts` - account of order where to dispatch amounts (part of whole)
async fn route(
    server: &str,
    input : BankInput,
    glt: GetConfigResponse,
) -> CvmProgram {
    let blackbox: Client = Client::new(server);
    let route = blackbox
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
        .get(0).expect("at least one route");    

    let mut program = CvmProgram::new();
    build_next(&mut current, &mut route.next);
    return program;   
}
