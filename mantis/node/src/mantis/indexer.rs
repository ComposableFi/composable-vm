use cvm_runtime::outpost::GetConfigResponse;
use cw_mantis_order::OrderItem;

use crate::mantis::cosmos::cosmwasm::smart_query;

use super::cosmos::client::{CosmWasmReadClient, Tip};

pub async fn get_active_orders(
    order_contract: &String,
    cosmos_query_client: &mut CosmWasmReadClient,
    tip: &Tip,
) -> Vec<OrderItem> {
    let query = cw_mantis_order::QueryMsg::GetAllOrders {};
    smart_query::<_, Vec<OrderItem>>(order_contract, query, cosmos_query_client)
        .await
        .into_iter()
        .filter(|x| x.msg.timeout > tip.block.value())
        .collect::<Vec<OrderItem>>()
}

pub async fn get_stale_orders(
    order_contract: &String,
    cosmos_query_client: &mut CosmWasmReadClient,
    tip: &Tip,
) -> Vec<OrderItem> {
    let query = cw_mantis_order::QueryMsg::GetAllOrders {};
    smart_query::<_, Vec<OrderItem>>(order_contract, query, cosmos_query_client)
        .await
        .into_iter()
        .filter(|x| x.msg.timeout < tip.block.value())
        .collect::<Vec<OrderItem>>()
}

pub async fn has_stale_orders(
    order_contract: &String,
    cosmos_query_client: &mut CosmWasmReadClient,
    tip: &Tip,
) -> bool {
    let query = cw_mantis_order::QueryMsg::HasStale {};
    smart_query::<_, bool>(order_contract, query, cosmos_query_client).await
}

pub async fn get_cvm_glt(
    contract: &String,
    cosmos_query_client: &mut CosmWasmReadClient,
) -> GetConfigResponse {
    let query = cvm_runtime::outpost::QueryMsg::GetConfig {};
    smart_query::<_, GetConfigResponse>(contract, query, cosmos_query_client).await
}
