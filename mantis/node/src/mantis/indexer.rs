use cvm_runtime::outpost::GetConfigResponse;
use cw_mantis_order::OrderItem;

use crate::mantis::cosmos::cosmwasm::smart_query;

use super::cosmos::client::{CosmWasmReadClient, Tip};


pub async fn get_all_orders(
    order_contract: &String,
    cosmos_query_client: &mut CosmWasmReadClient,
    tip: &Tip,
) -> Vec<OrderItem> {
    let query = cw_mantis_order::QueryMsg::GetAllOrders {};
    let all_orders = smart_query::<_, Vec<OrderItem>>(order_contract, query, cosmos_query_client)
        .await
        .into_iter()
        .filter(|x| x.msg.timeout > tip.block.value())
        .collect::<Vec<OrderItem>>();
    println!("all_orders: {:?}", all_orders);
    all_orders
}

pub async fn get_cvm_glt(contract: &String,  cosmos_query_client: &mut CosmWasmReadClient,
) -> GetConfigResponse {
    let query = cvm_runtime::outpost::QueryMsg::GetConfig {};
    let cvm_glt = smart_query::<_, GetConfigResponse>(contract, query, cosmos_query_client)
        .await;
    cvm_glt    
}