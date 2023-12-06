#![allow(clippy::disallowed_methods)] // does unwrap inside
#![allow(deprecated)] // sylvia macro

mod simulator;
mod events;
mod prelude;
mod state;
mod types;

use simulator::simulate_cows_via_bank;
use events::order::*;
use events::solution::*;
use prelude::*;
use simulator::simulate_route;
use state::*;
pub use types::*;

pub use crate::sv::{ExecMsg, QueryMsg};

use cosmwasm_std::{wasm_execute, Addr, BankMsg, Coin, Event, Order, StdError, Storage};
use cvm_runtime::shared::{XcInstruction, XcProgram};
use cw_storage_plus::{Index, IndexList, IndexedMap, Item, Map};
use sylvia::{
    contract,
    cw_std::{ensure, Response, StdResult},
    entry_points,
    types::{ExecCtx, InstantiateCtx, QueryCtx},
};

pub struct OrderContract<'a> {
    pub orders: Map<'a, u128, OrderItem>,
    /// (a,b,solver)
    pub solutions:
        IndexedMap<'a, &'a (Denom, Denom, SolverAddress), SolutionItem, SolutionIndexes<'a>>,
    pub next_order_id: Item<'a, u128>,
    /// address for CVM contact to send routes to
    pub cvm_address: Item<'a, String>,
    pub admin: cw_controllers::Admin<'a>,
}

impl Default for OrderContract<'_> {
    fn default() -> Self {
        Self {
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
            .expect("there are some funds in order");

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

    /// until order/solution in execution can cancel
    /// cancellation of order is delayed so solvers can observe it
    /// can remove up only my orders and solution
    #[msg(exec)]
    pub fn cancel(
        &self,
        _ctx: ExecCtx,
        orders: Vec<OrderId>,
        solution: Option<Addr>,
    ) -> StdResult<Response> {
        todo!("remove order and send event")
    }

    /// how it works

    /// 2. add additional table with in execution orders to bind them to solution
    /// 3. solution collects assets and send them to CVM
    /// 4. expected final step of CVM to transfer back to interpreter
    /// 5. and do RAW call with assets with SOLVE and 100% amounts
    /// 6. raw call dispatches amounts, and unlocks orders
    /// 7. please note that orders still can be solved cow while in cross chain


    #[msg(exec)]
    pub fn route(&self, ctx: ExecCtx, msg: RouteSubMsg) -> StdResult<Response> {
        ensure!(
            ctx.info.sender == ctx.env.contract.address,
            StdError::GenericErr {
                msg: "only self can call this".to_string()
            }
        );

        ctx.deps.api.debug(
            "so here we add route execution tracking to storage and map route to CVM program",
        );

        let cvm = cvm_runtime::gateway::ExecuteMsg::ExecuteProgram(
            cvm_runtime::gateway::ExecuteProgramMsg {
                salt: vec![],           // derived from solutffzion owner and block - which is id
                program: msg.route,     // traversed to set
                assets: <_>::default(), // collecting all assets remaining from orders
                tip: None,
            },
        );
        let contract = self.cvm_address.load(ctx.deps.storage)?;
        let cvm = wasm_execute(contract, &cvm, vec![])?;
        // in success handler of msg we start tracking solution
        // in failure handler we move all funds to users back
        Ok(Response::default().add_message(cvm))
    }

    /// Provides solution for set of orders.
    /// All fully
    #[msg(exec)]
    pub fn solve(&self, ctx: ExecCtx, msg: SolutionSubMsg) -> StdResult<Response> {
        // read all orders as solver provided
        let mut all_orders = self.join_solution_with_orders(&msg, &ctx)?;
        let at_least_one = all_orders.first().expect("at least one");

        // normalize pair
        let mut ab = (
            at_least_one.given().denom.clone(),
            at_least_one.wants().denom.clone(),
        );
        ab.sort_selection();

        // add solution to total solutions
        let possible_solution = SolutionItem {
            pair: ab.clone(),
            msg,
            block_added: ctx.env.block.height,
            owner: ctx.info.sender.clone(),
        };

        self.solutions.save(
            ctx.deps.storage,
            &(
                ab.clone().0,
                ab.clone().1,
                ctx.info.sender.clone().to_string(),
            ),
            &possible_solution,
        )?;
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

        // pick up optimal solution with solves with bank
        let mut a_in = 0u128;
        let mut b_in = 0u128;
        let (a, b) = ab.clone();
        let mut transfers = vec![];
        let mut solution_item: SolutionItem = possible_solution;
        for solution in all_solutions {
            let alternative_all_orders = self.join_solution_with_orders(&solution.msg, &ctx)?;
            let a_total_in: u128 = alternative_all_orders
                .iter()
                .filter(|x| x.given().denom == a)
                .map(|x: &SolvedOrder| x.given().amount.u128())
                .sum();
            let b_total_in: u128 = alternative_all_orders
                .iter()
                .filter(|x: &&SolvedOrder| x.given().denom == b)
                .map(|x| x.given().amount.u128())
                .sum();

            let cow_part =
                simulate_cows_via_bank(&alternative_all_orders.clone(), a_total_in, b_total_in);

            ctx.deps
                .api
                .debug(&format!("mantis::solutions::alternative {:?}", &cow_part));
            

            
            if let Err(err) = cow_part {
                if solution.owner == ctx.info.sender {
                    return Err(err);
                }
            } else if let Ok(cow_part) = cow_part {
                if let Some(route) = solution.msg.route {
                    let cvm = simulate_route(
                        ctx.deps.storage,
                        route,
                        Coin {
                            denom: ab.0.clone(),
                            amount: cow_part.token_a_remaining,
                        },
                        Coin {
                            denom: ab.1.clone(),
                            amount: cow_part.token_b_remaining,
                        },
                    );
                    
                }
                if a_total_in.saturating_mul(b_total_in) >= a_in.saturating_mul(b_in) {
                    a_in = a_total_in;
                    b_in = b_total_in;
                    all_orders = alternative_all_orders;
                    transfers = cow_part.filled;
                    solution_item = solution;
                }
            }
        }

        let mut response = Response::default();

        let transfers = self.fill(
            ctx.deps.storage,
            transfers,
            ctx.info.sender.to_string(),
            solution_item.block_added,
        )?;

        if let Some(route) = solution_item.msg.route {
            // send remaining for settlement
            let route = wasm_execute(
                ctx.env.contract.address,
                &ExecMsg::route(RouteSubMsg { all_orders, route }),
                vec![],
            )?;
            response = response.add_message(route);
        };

        self.solutions.clear(ctx.deps.storage);
        let solution_chosen = mantis_solution_chosen(ab, &ctx, &transfers);

        ctx.deps
            .api
            .debug(&format!("mantis::solution::chosen: {:?}", &solution_chosen));
        for transfer in transfers {
            response = response.add_message(transfer.bank_msg);
            response = response.add_event(transfer.event);
        }
        Ok(response
            .add_event(solution_upserted)
            .add_event(solution_chosen))
    }



    /// Simple get all orders
    #[msg(query)]
    pub fn get_all_orders(&self, ctx: QueryCtx) -> StdResult<Vec<OrderItem>> {
        self.orders
            .range_raw(ctx.deps.storage, None, None, Order::Ascending)
            .map(|r| r.map(|(_, order)| order))
            .collect::<StdResult<Vec<OrderItem>>>()
    }

    #[msg(query)]
    pub fn get_all_solutions(&self, ctx: QueryCtx) -> StdResult<Vec<SolutionItem>> {
        self.get_solutions(ctx.deps.storage)
    }

    fn get_solutions(&self, storage: &dyn Storage) -> Result<Vec<SolutionItem>, StdError> {
        self.solutions
            .idx
            .pair
            .range(storage, None, None, Order::Ascending)
            .map(|r| r.map(|(_, x)| x))
            .collect()
    }

    /// (partially) fills orders.
    /// Returns relevant transfers and sets proper tracking info for remaining cross chain
    /// execution. Orders which are in cross chain execution are "locked", users cannot cancel them
    /// or take funds back during execution (because funds are moved).
    fn fill(
        &self,
        storage: &mut dyn Storage,
        cows: Vec<CowFilledOrder>,
        solver_address: String,
        solution_block_added: u64,
    ) -> StdResult<Vec<CowFillResult>> {
        let mut results = vec![];
        for (transfer, order) in cows.into_iter() {
            let mut order: OrderItem = self.orders.load(storage, order.u128())?;
            order.fill(transfer.amount);
            let (event, remaining) = if order.given.amount.is_zero() {
                self.orders.remove(storage, order.order_id.u128());
                (mantis_order_filled_full(&order, &solver_address, solution_block_added), None)
            } else {
                self.orders.save(storage, order.order_id.u128(), &order)?;
                (
                mantis_order_filled_parts(&order, &transfer, &solver_address, solution_block_added),
                Some(order))
            };
            let transfer = BankMsg::Send {
                to_address: order.owner.to_string(),
                amount: vec![transfer],
            };
            results.push(CowFillResult {
                remaining,
                bank_msg: transfer,
                event,
            });
        }
        Ok(results)
    }
}

