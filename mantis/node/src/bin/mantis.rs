use cosmos_sdk_proto::cosmwasm::wasm::v1::QuerySmartContractStateRequest;

use cosmrs::{cosmwasm::MsgExecuteContract, rpc::Client};
use cw_mantis_order::OrderItem;
use mantis_node::{
    mantis::{args::*, cosmos::*},
    prelude::*,
};

#[tokio::main]
async fn main() {
    let args = MantisArgs::parsed();
    let read_client = create_wasm_query_client(&args.centauri).await;

    let signer = mantis_node::mantis::beaker::cli::support::signer::from_mnemonic(
        args.wallet.as_str(),
        "centauri",
    )
    .expect("mnemonic");




    let mut write_client = create_wasm_write_client(&args.centauri).await;

    loop {
        if let Some(assets) = args.simulate.clone() {
            simulate_order(&mut write_client, args.order_contract.clone(), assets).await;
        };
    }
}

/// `assets` - is comma separate list. each entry is amount u64 glued with alphanumeric denomination
/// that is splitted into array of CosmWasm coins.
/// one coin is chosen as given,
/// from remaining 2 other coins one is chosen as wanted
/// amount of count is randomized around values
///
/// `write_client`
/// `order_contract` - orders are formed for give and want, and send to orders contract.
/// timeout is also randomized starting from 10 to 100 blocks
///
/// Also calls `timeout` so old orders are cleaned.
async fn simulate_order(write_client: &mut CosmWasmWriteClient, order_contract: String, asset: String, key : cosmrs::crypto::secp256k1::SigningKey) {
    if std::time::Instant::now().elapsed().as_millis() % 100 == 0 {
        
        let msg = MsgExecuteContract {
            sender: key.public_key(),
            contract: todo!(),
            msg: todo!(),
            funds: todo!(),
        };

        //let result = write_client.execute_contract(request).await.expect("executed");
    }
}

/// gets orders, groups by pairs
/// solves them using algorithm
/// if any volume solved, posts solution
///
/// gets data from chain pools/fees on osmosis and neutron
/// gets CVM routing data
/// uses cfmm algorithm
async fn solve(
    read: &mut CosmWasmReadClient,
    _write: CosmWasmWriteClient,
    order_contract: String,
    _cvm_contract: String,
) {
    let query = cw_mantis_order::QueryMsg::GetAllOrders {};
    let orders_request = QuerySmartContractStateRequest {
        address: order_contract.clone(),
        query_data: serde_json_wasm::to_vec(&query).expect("json"),
    };
    let orders = read
        .smart_contract_state(orders_request)
        .await
        .expect("orders obtained")
        .into_inner()
        .data;
    let orders: Vec<OrderItem> = serde_json_wasm::from_slice(&orders).expect("orders");

    let orders = orders.into_iter().group_by(|x| {
        if x.given.denom < x.msg.wants.denom {
            (x.given.denom.clone(), x.msg.wants.denom.clone())
        } else {
            (x.msg.wants.denom.clone(), x.given.denom.clone())
        }
    });
    for (pair, orders) in orders.into_iter() {
        // solve here !
        // post solution
        // just print them for now
        println!("pair {pair:?} orders: {:?}", orders.collect::<Vec<_>>());
    }
}
