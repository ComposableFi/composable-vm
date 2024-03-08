/// Given total amount it, order owners and desired out, produce CVM program from and by requesting route

use cvm_runtime::{shared::XcProgram, AssetId, ExchangeId};
use blackbox_rs::{prelude::*, Client};

async fn route(server: &str, in_asset_id: AssetId, in_asset_amount: AssetId, out_asset_id: AssetId) -> XcProgram {
    let blackbox: Client = Client::new(server);
    let route = blackbox.simulator_router_simulator_router_get(
        in_asset_id,
        in_asset_amount,
        true,
        out_asset_id,
        10
    ).await.unwrap();
    
    panic!()
}