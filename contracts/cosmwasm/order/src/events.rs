pub mod order {

    use crate::{prelude::*, TrackedOrderItem};
    use cosmwasm_std::Event;

    use crate::OrderItem;

    pub fn order_created(order_id: u128, order: &OrderItem) -> Event {
        Event::new("mantis-order-created")
            .add_attribute("order_id", order_id.to_string())
            .add_attribute("order_given_amount", order.given.amount.to_string())
            .add_attribute("order_given_denom", order.given.denom.to_string())
            .add_attribute("order_owner", order.owner.to_string())
            .add_attribute("order_wants_amount", order.msg.wants.amount.to_string())
            .add_attribute("order_wants_denom", order.msg.wants.denom.to_string())
    }

    pub fn mantis_order_filled_partially(
        order: &OrderItem,
        transfer: &Coin,
        solver_address: &String,
        solution_block_added: u64,
    ) -> Event {
        Event::new("mantis-order-filled-parts")
            .add_attribute("order_id", order.order_id.to_string())
            .add_attribute("amount", transfer.amount.to_string())
            .add_attribute("solver_address", solver_address.to_owned())
            .add_attribute("solution_block_added", solution_block_added.to_string())
    }

    pub fn mantis_order_filled_full(
        order: &OrderItem,
        solver_address: &str,
        solution_block_added: u64,
    ) -> Event {
        Event::new("mantis-order-filled-full")
            .add_attribute("order_id", order.order_id.to_string())
            .add_attribute("solver_address", solver_address)
            .add_attribute("solution_block_added", solution_block_added.to_string())
    }

    pub fn mantis_order_routed_full(order: &OrderItem, solver_address: &String) -> Event {
        Event::new("mantis-order-routed-full")
            .add_attribute("order_id", order.order_id.to_string())
            .add_attribute("solver_address", solver_address.clone())
    }

    pub fn mantis_order_cross_chain_tracked(tracking: &TrackedOrderItem) -> Event {
        Event::new("mantis-order-cross-chain-tracked")
            .add_attribute("order_id", tracking.order_id.to_string())
            .add_attribute("amount_taken", tracking.amount_taken.to_string())
            .add_attribute("promised", tracking.promised.to_string())
            .add_attribute(
                "meta",
                "solution is not verified, do not use for big funds".to_string(),
            )
    }
}

pub mod solution {
    use crate::{prelude::*, Block, CowFillResult, DenomPair};
    use cosmwasm_std::Event;
    use sylvia::types::ExecCtx;

    pub fn mantis_solution_chosen(
        ab: DenomPair,
        ctx: &ExecCtx<'_>,
        transfers: &[CowFillResult],
        cow_volume: u128,
        cross_chain_volume: u128,
        owner: Addr,
        block_added: Block,
    ) -> Event {
        let solution_id = crate::types::solution_id(&(owner.to_string(), ab.clone(), block_added));

        Event::new("mantis-solution-chosen")
            .add_attribute("token_a", ab.clone().a)
            .add_attribute("token_b", ab.clone().b)
            .add_attribute("solver_address", ctx.info.sender.to_string())
            .add_attribute("cow_volume", cow_volume.to_string())
            .add_attribute("cross_chain_volume", cross_chain_volume.to_string())
            .add_attribute("total_transfers", transfers.len().to_string())
            .add_attribute("solution_id", hex::encode(solution_id))
            .add_attribute("solution_block_added", block_added.to_string())
    }

    pub fn mantis_solution_upserted(ab: &DenomPair, ctx: &ExecCtx<'_>) -> Event {
        Event::new("mantis-solution-upserted")
            .add_attribute("token_a", ab.clone().a)
            .add_attribute("token_b", ab.clone().b)
            .add_attribute("solver_address", ctx.info.sender.to_string())
    }
}
