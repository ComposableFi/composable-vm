use blackbox_rs::{prelude::*, types::*, Client};
/// Given total amount it, order owners and desired out, produce CVM program from and by requesting route
use cvm_runtime::{
    outpost::GetConfigResponse, shared::{CvmInstruction, Displayed, XcAddr, CvmProgram}, Amount, AssetId, ExchangeId 
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

        fn build_next(current: CvmProgram, next: &mut [NextItem]) -> CvmInstruction {
            match next.split_first_mut() {
                Some((head, rest) => {
                    match head {
                        NextItem::Exchange(_) => todo!(),
                        NextItem::Spawn(_) => todo!(),
                    }
                }
                None => info!("no more routes"),
            }
        }
    }
    CvmInstruction::Spawn { network_id: (), salt: (), assets: (), program: () }   
}
