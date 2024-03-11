use blackbox_rs::{prelude::*, types::*, Client};
/// Given total amount it, order owners and desired out, produce CVM program from and by requesting route
use cvm_runtime::{
    shared::{Displayed, XcProgram},
    Amount, AssetId, ExchangeId,
};

async fn route(
    server: &str,
    in_asset_id: AssetId,
    in_asset_amount: Displayed<u64>,
    out_asset_id: AssetId,
) -> XcProgram {
    let blackbox: Client = Client::new(server);
    let route = blackbox
        .simulator_router_simulator_router_get(
            &InAssetAmount::Variant0(in_asset_amount.0.try_inot().expect("in_asset_amount")),
            in_asset_id.to_string().into(),
            true,
            out_asset_id.to_string().into(),
            10.into(),
        )
        .await
        .unwrap();

    panic!()
}
