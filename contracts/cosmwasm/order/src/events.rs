fn order_created(order_id: u128, order: &OrderItem) -> Event {
    Event::new("mantis-order-created")
        .add_attribute("order_id", order_id.to_string())
        .add_attribute("order_given_amount", order.given.amount.to_string())
        .add_attribute("order_given_denom", order.given.denom.to_string())
        .add_attribute("order_owner", order.owner.to_string())
        .add_attribute("order_wants_amount", order.msg.wants.amount.to_string())
        .add_attribute("order_wants_denom", order.msg.wants.denom.to_string())
}