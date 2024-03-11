use blackbox_rs::{prelude::*, types::*, Client};
/// Given total amount it, order owners and desired out, produce CVM program from and by requesting route
use cvm_runtime::{
    shared::{Displayed, XcProgram, XcAddr},
    Amount, AssetId, ExchangeId, 
};

/// `order_accounts` - account of order where to dispatch amounts (part of whole)
async fn route(
    server: &str,
    in_asset_id: AssetId,
    in_asset_amount: Displayed<u64>,
    out_asset_id: AssetId,
    order_accounts: Vec<(XcAddr, Amount)>,
) -> XcProgram {
    let blackbox: Client = Client::new(server);
    let route = blackbox
        .simulator_router_simulator_router_get(
            &InAssetAmount::Variant0(in_asset_amount.0.try_into().expect("in_asset_amount")),
            &InAssetId::Variant1(in_asset_id.to_string()),
            true,
            &OutAssetAmount::Variant0(10),
            &OutAssetId::Variant1(out_asset_id.to_string().into()),
        )
        .await
        .expect("route found");
    
    panic!()
}
