async fn get_all_orders(
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
