
pub mod order {

    pub fn order_created(order_id: u128, order: &OrderItem) -> Event {
        Event::new("mantis-order-created")
            .add_attribute("order_id", order_id.to_string())
            .add_attribute("order_given_amount", order.given.amount.to_string())
            .add_attribute("order_given_denom", order.given.denom.to_string())
            .add_attribute("order_owner", order.owner.to_string())
            .add_attribute("order_wants_amount", order.msg.wants.amount.to_string())
            .add_attribute("order_wants_denom", order.msg.wants.denom.to_string())
    }

    pub fn mantis_order_filled_parts(order: &OrderItem, transfer: &Coin, solver_address: &String, solution_block_added: u64) -> Event {
        Event::new("mantis-order-filled-parts")
            .add_attribute("order_id", order.order_id.to_string())
            .add_attribute("amount", transfer.amount.to_string())
            .add_attribute("solver_address", solver_address.clone())
            .add_attribute("solution_block_added", solution_block_added.to_string())
    }
    
    
    pub fn mantis_order_filled_full(order: &OrderItem, solver_address: &String, solution_block_added: u64) -> Event {
        Event::new("mantis-order-filled-full")
            .add_attribute("order_id", order.order_id.to_string())
            .add_attribute("solver_address", solver_address.clone())
            .add_attribute("solution_block_added", solution_block_added.to_string())
    }
}

pub mod solution {
    use cosmwasm_std::Event;


    pub fn mantis_solution_chosen(ab: Pair, ctx: &ExecCtx<'_>, transfers: &Vec<CowFillResult>, cow_volume : u128, cross_chain_volume : u128) -> Event {
        let solution_chosen = Event::new("mantis-solution-chosen")
            .add_attribute("token_a", ab.clone().0)
            .add_attribute("token_b", ab.clone().1)
            .add_attribute("solver_address", ctx.info.sender.to_string())
            .add_attribute("cow_volume", cow_volume.to_string())
            .add_attribute("cross_chain_volume", cow_volume.to_string())
            .add_attribute("total_transfers", transfers.len().to_string());
        solution_chosen
    }

    pub fn mantis_solution_upserted(ab: &Pair, ctx: &ExecCtx<'_>) -> Event {
        let solution_upserted = Event::new("mantis-solution-upserted")
            .add_attribute("token_a", ab.clone().0)
            .add_attribute("token_b", ab.clone().1)
            .add_attribute("solver_address", ctx.info.sender.to_string());
        solution_upserted
    }    
}



