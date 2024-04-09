#![allow(clippy::disallowed_methods)] // does unwrap inside
#![allow(deprecated)] // sylvia macro

mod constants;
mod errors;
mod events;
mod prelude;
mod simulator;
mod state;
mod types;
mod validation;

use constants::MIN_SOLUTION_COUNT;
use events::order::*;
use events::solution::*;
use itertools::Itertools;
use mantis_cw::DenomPair;
use prelude::*;
use simulator::simulate_cows_via_bank;
use state::*;
pub use types::*;

pub use crate::sv::{ExecMsg, QueryMsg};

use cosmwasm_std::{wasm_execute, Addr, BankMsg, Coin, Event, Order, StdError, Storage};
use cvm_runtime::shared::CvmProgram;
use cw_storage_plus::{Index, IndexList, IndexedMap, Item, Map};
use sylvia::{
    contract,
    cw_std::{ensure, Response, StdResult},
    entry_points,
    types::{ExecCtx, InstantiateCtx, QueryCtx},
};

pub struct OrderContract<'a> {
    pub orders: Map<'a, u128, OrderItem>,

    pub tracked_orders: Map<'a, (u128, SolutionHash), TrackedOrderItem>,
    pub solutions: SolutionMultiMap<'a>,
    pub next_order_id: Item<'a, u128>,
    /// address for CVM contact to send routes to
    pub cvm_address: Item<'a, String>,
    pub admin: cw_controllers::Admin<'a>,
    /// block on which pair got some solution
    pub pair_to_block: Map<'a, DenomPair, Block>,
}

impl Default for OrderContract<'_> {
    fn default() -> Self {
        Self {
            pair_to_block: Map::new("pair_to_block"),
            tracked_orders: Map::new("tracked_orders"),
            orders: Map::new("orders"),
            next_order_id: Item::new("next_order_id"),
            cvm_address: Item::new("cvm_address"),
            solutions: solutions(),
            admin: cw_controllers::Admin::new("admin"),
        }
    }
}

#[entry_points]
#[contract]
impl OrderContract<'_> {
    pub fn new() -> Self {
        Self::default()
    }
    #[msg(instantiate)]
    pub fn instantiate(
        &self,
        mut ctx: InstantiateCtx,
        admin: Option<Addr>,
        cvm_address: Addr,
    ) -> StdResult<Response> {
        self.cvm_address
            .save(ctx.deps.storage, &cvm_address.into_string())?;
        self.admin
            .set(ctx.deps.branch(), Some(admin.unwrap_or(ctx.info.sender)))?;
        Ok(Response::default())
    }

    /// This contracts receives user order, takes ddos protection deposit (to protect solvers from
    /// spamming), and stores order for searchers.
    #[msg(exec)]
    pub fn order(&self, ctx: ExecCtx, msg: OrderSubMsg) -> StdResult<Response> {
        // for now we just use bank for ics20 tokens
        let funds = ctx
            .info
            .funds
            .first()
            .ok_or(errors::expected_some_funds_in_order())?;

        // just save order under incremented id
        let order_id = self
            .next_order_id
            .load(ctx.deps.storage)
            .unwrap_or_default();
        let order = OrderItem {
            msg,
            given: funds.clone(),
            order_id: order_id.into(),
            owner: ctx.info.sender,
        };
        self.orders.save(ctx.deps.storage, order_id, &order)?;
        self.next_order_id.save(ctx.deps.storage, &(order_id + 1))?;
        let order_created = order_created(order_id, &order);
        ctx.deps
            .api
            .debug(&format!("mantis::order::created: {:?}", order));
        Ok(Response::default().add_event(order_created))
    }

    /// Hook/crank for cleanup.
    /// Caller receives small reward for doing so.
    /// This is to prevent spamming of old orders.
    /// If input collections are empty, one clean ups ALL orders
    #[msg(exec)]
    pub fn timeout(
        &self,
        ctx: ExecCtx,
        orders: Vec<OrderId>,
        solutions: Vec<Addr>,
    ) -> StdResult<Response> {
        ctx.deps.api.debug(&format!(
            "mantis::timeout::cleaning orders {:?} and solutions {:?}",
            orders, solutions
        ));
        let orders: Result<Vec<(u128, OrderItem)>, _> = self
            .orders
            .range(ctx.deps.storage, None, None, Order::Ascending)
            .filter(|x| {
                let (_id, order) = x.as_ref().unwrap();
                order.msg.timeout < ctx.env.block.height
            })
            .collect();
        let mut msgs = vec![];
        for order in orders? {
            self.orders.remove(ctx.deps.storage, order.0);
            let msg = BankMsg::Send {
                amount: [order.1.given].to_vec(),
                to_address: order.1.owner.to_string(),
            };
            msgs.push(msg);
        }
        Ok(Response::default().add_messages(msgs))
    }

    /// Allow admin to move stuck funds.
    #[msg(exec)]
    pub fn force_transfer(
        &self,
        ctx: ExecCtx,
        amount: Vec<Coin>,
        to_address: String,
    ) -> StdResult<Response> {
        ensure!(
            self.admin.is_admin(ctx.deps.as_ref(), &ctx.info.sender)?,
            StdError::generic_err("only admin can call this")
        );
        let msg: BankMsg = BankMsg::Send { to_address, amount };
        Ok(Response::default().add_message(msg))
    }

    /// Until order/solution in execution can cancel.
    /// Cancellation of order is delayed so solvers can observe it.
    /// Can remove up only my orders and solution.
    #[msg(exec)]
    pub fn cancel(
        &self,
        _ctx: ExecCtx,
        _orders: Vec<OrderId>,
        _solution: Option<Addr>,
    ) -> StdResult<Response> {
        todo!("remove order and send event")
    }

    #[msg(exec)]
    pub fn route(&self, mut ctx: ExecCtx, msg: RouteSubMsg) -> StdResult<Response> {
        ensure!(
            ctx.info.sender == ctx.env.contract.address
                || ctx.info.sender
                    == self
                        .admin
                        .query_admin(ctx.deps.as_ref())?
                        .admin
                        .unwrap_or_default(),
            StdError::GenericErr {
                msg: "only self can call this".to_string()
            }
        );

        ctx.deps.api.debug(
            &format!("mantis::route:: so here we add route execution tracking to storage and map route to CVM program {:?}", msg)
        );

        let cvm: cvm_runtime::outpost::ExecuteMsg =
            cvm_runtime::outpost::ExecuteMsg::ExecuteProgram(msg.msg.msg);

        let contract = self.cvm_address.load(ctx.deps.storage)?;

        let cvm_filled = self.pre_fill_remotely(
            ctx.branch(),
            msg.msg.optimal_price,
            msg.solver_address,
            msg.solution_id,
            msg.solved_orders,
            msg.pair.clone(),
        )?;

        let a_funds = cvm_filled
            .iter()
            .filter(|x| x.tracking.amount_taken.denom == msg.pair.a)
            .map(|x| x.tracking.amount_taken.amount)
            .sum();

        let b_funds = cvm_filled
            .iter()
            .filter(|x| x.tracking.amount_taken.denom == msg.pair.b)
            .map(|x| x.tracking.amount_taken.amount)
            .sum();

        let funds = vec![
            Coin {
                denom: msg.pair.a,
                amount: a_funds,
            },
            Coin {
                denom: msg.pair.b,
                amount: b_funds,
            },
        ];

        let events = cvm_filled
            .iter()
            .map(|x| mantis_order_cross_chain_tracked(&x.tracking));

        let cvm = wasm_execute(contract, &cvm, funds)?;
        Ok(Response::default().add_message(cvm).add_events(events))
    }

    /// Executes single order via CVM without use of CoW
    #[msg(exec)]
    pub fn execute(
        &self,
        ctx: ExecCtx,
        order_id: OrderId,
        cvm_program: CvmProgram,
    ) -> StdResult<Response> {
        let order: OrderItem = self.orders.load(ctx.deps.storage, order_id.u128())?;
        validation::validate_solver(ctx.deps.as_ref(), &ctx.info.sender, &order)?;
        self.orders.remove(ctx.deps.storage, order_id.u128());
        let cvm = wasm_execute(
            self.cvm_address.load(ctx.deps.storage)?,
            &cvm_program,
            vec![order.given],
        )?;
        Ok(Response::default().add_message(cvm))
    }

    /// Provides solution for set of orders.
    /// All fully
    #[msg(exec)]
    pub fn solve(&self, mut ctx: ExecCtx, msg: SolutionSubMsg) -> StdResult<Response> {
        // read all orders as solver provided
        let mut all_orders = join_solution_with_orders(&self.orders, &msg, &ctx)?;
        let at_least_one = all_orders.first().expect("at least one");

        // normalize pair
        let ab = DenomPair::new(
            at_least_one.given().denom.clone(),
            at_least_one.wants().denom.clone(),
        );

        // add solution to total solutions
        let possible_solution = SolutionItem {
            pair: ab.clone(),
            msg,
            block_added: ctx.env.block.height,
            owner: ctx.info.sender.clone(),
        };

        self.start_solution(ctx.branch(), ab.clone())?;

        let solution_key = (ab.clone(), ctx.info.sender.clone().to_string());

        self.solutions
            .save(ctx.deps.storage, &solution_key, &possible_solution)?;
        let solution_upserted = mantis_solution_upserted(&ab, &ctx);

        ctx.deps.api.debug(&format!(
            "mantis::solution::upserted {:?}",
            &solution_upserted
        ));

        // get all solution for pair
        let all_solutions: Result<Vec<SolutionItem>, _> = self
            .solutions
            .prefix(ab.clone())
            .range(ctx.deps.storage, None, None, Order::Ascending)
            .map(|r| r.map(|(_, solution)| solution))
            .collect();
        let all_solutions = all_solutions?;
        ctx.deps
            .api
            .debug(&format!("mantis::solutions::current {:?}", all_solutions));

        // wait before solve
        let started = self
            .pair_to_block
            .load(ctx.deps.storage, ab.clone())
            .expect("pair to block");

        if started + constants::BATCH_EPOCH as u64 > ctx.env.block.height {
            return Ok(Response::default());
        } else {
            self.pair_to_block.remove(ctx.deps.storage, ab.clone());
        }

        // pick up optimal solution with solves with bank
        let mut a_in = 0u128;
        let mut b_in = 0u128;
        let mut transfers = vec![];
        let mut solution_item: SolutionItem = possible_solution;
        let mut volume = 0u128;
        if all_solutions.len() < MIN_SOLUTION_COUNT as usize {
            return Ok(Response::default());
        }

        for solution in all_solutions {
            if validation::validate_solvers(&ctx.deps, &solution, &all_orders).is_err() {
                continue;
            }
            if validation::validate_routes(&ctx.deps, &solution, &all_orders).is_err() {
                continue;
            }
            let solution_orders = join_solution_with_orders(&self.orders, &solution.msg, &ctx)?;
            let a_total_from_orders_in_solution: u128 = solution_orders
                .iter()
                .filter(|x| x.given().denom == ab.a)
                .map(|x: &SolvedOrder| x.given().amount.u128())
                .sum();
            let b_total_from_orders_in_solution: u128 = solution_orders
                .iter()
                .filter(|x: &&SolvedOrder| x.given().denom == ab.b)
                .map(|x| x.given().amount.u128())
                .sum();

            let cow_part = simulate_cows_via_bank(
                &solution_orders.clone(),
                a_total_from_orders_in_solution,
                b_total_from_orders_in_solution,
            );

            ctx.deps
                .api
                .debug(&format!("mantis::solutions::alternative {:?}", &cow_part));

            if let Err(err) = cow_part {
                if solution.owner == ctx.info.sender {
                    return Err(err);
                }
            } else if let Ok(cow_part) = cow_part {
                let new_volume =
                    a_total_from_orders_in_solution.saturating_mul(b_total_from_orders_in_solution);
                if new_volume >= a_in.saturating_mul(b_in) {
                    a_in = a_total_from_orders_in_solution;
                    b_in = b_total_from_orders_in_solution;
                    all_orders = solution_orders;
                    transfers = cow_part.filled;
                    solution_item = solution;
                    volume = new_volume;
                }
            }
        }

        let solution_id = solution_item.id();
        let mut response = Response::default();

        let after_cows = self.fill_local(
            ctx.deps.storage,
            transfers,
            ctx.info.sender.to_string(),
            solution_item.block_added,
            solution_item.msg.optimal_price,
            &mut all_orders,
        )?;

        let cross_chain_b: u128 = all_orders
            .iter()
            .filter(|x| x.given().denom != ab.a)
            .map(|x| x.given_cross_chain().u128())
            .sum();

        let cross_chain_a: u128 = all_orders
            .iter()
            .filter(|x| x.given().denom != ab.b)
            .map(|x| x.given_cross_chain().u128())
            .sum();

        self.solutions.clear(ctx.deps.storage);
        let solution_chosen = emit_solution_chosen(
            ab.clone(),
            ctx.branch(),
            &after_cows,
            volume,
            cross_chain_b,
            cross_chain_a,
            solution_item.clone(),
        );

        for transfer in after_cows {
            response = response.add_message(transfer.bank_msg);
            response = response.add_event(transfer.event);
        }

        if let Some(msg) = solution_item.msg.route {
            let msg = wasm_execute(
                ctx.env.contract.address.clone(),
                &ExecMsg::route(RouteSubMsg {
                    solved_orders: all_orders,
                    msg,
                    solution_id,
                    pair: ab.clone(),
                    solver_address: solution_item.owner.to_string(),
                }),
                vec![],
            )?;
            response = response.add_message(msg);
        };

        Ok(response
            .add_event(solution_upserted)
            .add_event(solution_chosen))
    }

    fn start_solution(&self, ctx: ExecCtx<'_>, ab: DenomPair) -> Result<(), StdError> {
        if self
            .pair_to_block
            .load(ctx.deps.storage, ab.clone())
            .is_err()
        {
            self.pair_to_block
                .save(ctx.deps.storage, ab, &ctx.env.block.height)?;
        };
        Ok(())
    }

    /// Simple get all orders
    #[msg(query)]
    pub fn get_all_orders(&self, ctx: QueryCtx) -> StdResult<Vec<OrderItem>> {
        self.orders
            .range_raw(ctx.deps.storage, None, None, Order::Ascending)
            .map(|r| r.map(|(_, order)| order))
            .collect()
    }

    #[msg(query)]
    pub fn get_all_solutions(&self, ctx: QueryCtx) -> StdResult<Vec<SolutionItem>> {
        get_solutions(&self.solutions, ctx.deps.storage)
    }

    #[msg(query)]
    pub fn solution_id(&self, _ctx: QueryCtx, id: CrossChainSolutionKey) -> StdResult<String> {
        Ok(hex::encode(solution_id(&id)))
    }

    #[msg(query)]
    pub fn get_all_tracked_orders(&self, ctx: QueryCtx) -> StdResult<Vec<TrackedOrderItem>> {
        self.tracked_orders
            .range_raw(ctx.deps.storage, None, None, Order::Ascending)
            .map(|r| r.map(|(_, order)| order))
            .collect()
    }

    /// (partially) fills orders.
    /// Returns relevant transfers and sets proper tracking info for remaining cross chain
    /// execution. Orders which are in cross chain execution are "locked", users cannot cancel them
    /// or take funds back during execution (because funds are moved).
    fn fill_local(
        &self,
        storage: &mut dyn Storage,
        cows: Vec<CowFilledOrder>,
        solver_address: String,
        solution_block_added: u64,
        optimal_price: Ratio,
        solver_orders: &mut [SolvedOrder],
    ) -> StdResult<Vec<CowFillResult>> {
        let mut results = vec![];
        for (transfer, order) in cows.into_iter() {
            let mut order: OrderItem = self.orders.load(storage, order.u128())?;
            order.fill(transfer.amount, optimal_price)?;
            let (event, remaining) = if order.given.amount.is_zero() {
                // hey, need some other data structure for this
                let (_idx, solver_order) = solver_orders
                    .iter()
                    .find_position(|x| x.order.order_id == order.order_id)
                    .expect("solver order");

                ensure!(
                    solver_order.solution.cross_chain_part.is_none(),
                    errors::filled_order_cannot_be_cross_chain_routed()
                );

                self.orders.remove(storage, order.order_id.u128());
                (
                    mantis_order_filled_full(&order, &solver_address, solution_block_added),
                    false,
                )
            } else {
                let solver_order = solver_orders
                    .iter_mut()
                    .find(|x| x.order.order_id == order.order_id)
                    .expect("solver order");

                solver_order.order = order.clone();

                self.orders.save(storage, order.order_id.u128(), &order)?;
                (
                    mantis_order_filled_partially(
                        &order,
                        &transfer,
                        &solver_address,
                        solution_block_added,
                    ),
                    true,
                )
            };
            let transfer = BankMsg::Send {
                to_address: order.owner.to_string(),
                amount: vec![transfer],
            };
            results.push(CowFillResult {
                remaining: if remaining { Some(order) } else { None },
                bank_msg: transfer,
                event,
            });
        }
        Ok(results)
    }

    /// similar to `fill_local`, but instead of transfers via bank,
    /// produced movement of movement funds to tracking,
    /// but eventing and cleanup has same behavior
    fn pre_fill_remotely(
        &self,
        ctx: ExecCtx<'_>,
        _optimal_price: Ratio,
        solver_address: String,
        solution_id: SolutionHash,
        solver_orders: Vec<SolvedOrder>,
        _pair: DenomPair,
    ) -> Result<Vec<CvmFillResult>, StdError> {
        let mut result = vec![];
        for order in solver_orders.iter() {
            if let Some(cross_chain_part) = order.solution.cross_chain_part {
                match cross_chain_part {
                    OrderAmount::Part(_, _) => {
                        return Err(errors::partial_cross_chain_not_implemented())
                    }
                    OrderAmount::All => {
                        let event = mantis_order_routed_full(&order.order, &solver_address);
                        let tracker = TrackedOrderItem {
                            order_id: order.order.order_id,
                            solution_id: solution_id.clone(),
                            amount_taken: order.given().clone(),
                            // so we expect promised to be same as remaining.
                            // no really flexible
                            // really it must allow CoWs to violate limits,
                            // fixed by CVM later
                            // but bruno described differently, so we stick with that
                            promised: order.wants().amount,
                        };
                        self.orders
                            .remove(ctx.deps.storage, order.order.order_id.u128());
                        result.push(CvmFillResult::new(tracker, event));
                    }
                }
            }
        }
        Ok(result)
    }
}

fn emit_solution_chosen(
    ab: DenomPair,
    ctx: ExecCtx<'_>,
    transfers: &Vec<CowFillResult>,
    volume: u128,
    cross_chain_b: u128,
    cross_chain_a: u128,
    solution_item: SolutionItem,
) -> Event {
    let solution_chosen = mantis_solution_chosen(
        ab,
        &ctx,
        transfers,
        volume,
        cross_chain_b * cross_chain_a,
        solution_item.owner,
        solution_item.block_added,
    );

    ctx.deps
        .api
        .debug(&format!("mantis::solution::chosen: {:?}", &solution_chosen));
    solution_chosen
}
