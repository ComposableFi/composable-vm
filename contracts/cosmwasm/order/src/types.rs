use cosmwasm_std::{ensure, BankMsg, Event, StdError, StdResult, Uint64};
use cvm::{instruction::ExchangeId, network::NetworkId};

use crate::prelude::*;

pub type OrderId = Uint128;

pub type Amount = Uint128;

/// block moment (analog of timestamp)
pub type Block = u64;

/// each CoW solver locally, is just transfer from shared pair bank with referenced order
pub type CowFilledOrder = (Coin, OrderId);

/// each pair waits ate least this amount of blocks before being decided
pub const BATCH_EPOCH: u32 = 1;

/// count of solutions at minimum which can be decided, just set 1 for ease of devtest
pub const MIN_SOLUTION_COUNT: u32 = 1;

/// parts of a whole, numerator / denominator
pub type Ratio = (Uint64, Uint64);

#[cw_serde]
pub struct OrderItem {
    pub owner: Addr,
    pub msg: OrderSubMsg,
    pub given: Coin,
    pub order_id: OrderId,
}

impl OrderItem {
    pub fn fill(&mut self, transfer: &Coin) {
        self.msg.wants.amount -= transfer.amount;
        self.given.amount -= transfer.amount * self.given.amount / self.msg.wants.amount;
    }
}

#[cw_serde]
pub struct OrderSubMsg {
    /// Amount is minimum amount to get for given amount (sure user wants more than `wants` and we
    /// try to achieve that). Denom users wants to get, it can be cw20, bank or this chain CVM
    /// asset identifier. Only local CVM identifiers are accepted.
    /// If target asset identifier on other chain, use `transfer` to identity it.
    /// Why this is the case? It allows to CoW with user wanted assets which is not on
    /// settlement(this) chain.
    pub wants: Coin,

    /// How offchain SDK must work with it?
    /// ```example
    /// Alice gives token 42 on this(settlement chain).
    /// But she wants token 123 on other chain.
    /// SDK reads all CVM configurations.
    /// And tells Alice that there are 2 routes of asset 123 to/from settlement chain.
    /// These routes are 666 and 777. Each asset has unique route to settlement chain in CVM configuration.
    /// Alice picks route 777.
    /// So SDK sends 42 token as given to  and 777 as wanted,
    /// but additionally with attached transfer route Alice picked.  
    /// ```
    /// This allow to to CoWs for assets not on this chain.
    pub transfer: Option<TransferRoute>,
    /// how much blocks to wait for solution, if none, then cleaned up
    pub timeout: Block,
    /// if ok with partial fill, what is the minimum amount
    pub min_fill: Option<Ratio>,
}

#[cw_serde]
pub struct SolutionItem {
    pub pair: (String, String),
    pub msg: SolutionSubMsg,
    /// at which block solution was added
    pub block_added: u64,
}

/// price information will not be used on chain or deciding.
/// it will fill orders on chain as instructed
/// and check that max/min from orders respected
/// and sum all into volume. and compare solutions.
/// on chain cares each user gets what it wants and largest volume solution selected.
#[cw_serde]
pub struct SolutionSubMsg {
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub cows: Vec<Cow>,
    /// must adhere Connection.fork_join_supported, for now it is always false (it restrict set of
    /// routes possible)
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub route: Option<ExchangeRoute>,

    /// after some time, solver will not commit to success
    pub timeout: Block,
}

/// after cows solved, need to route remaining cross chain
#[cw_serde]
pub struct RouteSubMsg {
    pub all_orders: Vec<SolvedOrder>,
    pub route: ExchangeRoute,
}

/// how much of order to be solved by CoW.
/// difference with `Fill` to be solved by cross chain exchange
/// aggregate pool of all orders in solution is used to give user amount he wants.
#[cw_serde]
pub struct Cow {
    pub order_id: OrderId,
    /// how much of order to be solved by from bank for all aggregated cows
    pub cow_amount: Amount,
    /// amount of order to be taken (100% in case of full fill, can be less in case of partial)
    pub taken: Option<Amount>,
    /// amount user should get after order executed
    pub given: Amount,
}

#[cw_serde]
pub struct SolvedOrder {
    pub order: OrderItem,
    pub solution: Cow,
}

impl SolvedOrder {
    pub fn new(order: OrderItem, solution: Cow) -> StdResult<Self> {
        ensure!(
            order.msg.wants.amount <= solution.given,
            StdError::generic_err(format!(
                "user limit was not satisfied {order:?} {solution:?}"
            ))
        );

        Ok(Self { order, solution })
    }

    pub fn cross_chain(&self) -> u128 {
        self.order.msg.wants.amount.u128() - self.solution.cow_amount.u128()
    }

    pub fn given(&self) -> &Coin {
        &self.order.given
    }

    pub fn wants(&self) -> &Coin {
        &self.order.msg.wants
    }

    pub fn owner(&self) -> &Addr {
        &self.order.owner
    }
}

/// Route which may spawn on the way.
#[cw_serde]
pub struct ExchangeRoute {
    // on this chain
    pub exchanges: Vec<Exchange>,
    pub spawns: Vec<Spawn<ExchangeRoute>>,
}

/// Purely transfer route.
#[cw_serde]
pub struct TransferRoute {
    pub spawn: Vec<Spawn<TransferRoute>>,
}

/// Abstracted out route of underlying encoding on specific transport.
/// In the end of route, amount is always put onto user CVM executor.
#[cw_serde]
pub struct Spawn<Route> {
    pub to_chain: NetworkId,
    pub carry: Vec<Uint128>,
    pub execute: Option<Route>,
}

#[cw_serde]
pub struct Exchange {
    pub pool_id: ExchangeId,
    pub give: Uint128,
    pub want_min: Uint128,
}

/// when solution is applied to order item,
/// what to ask from host to do next
pub struct CowFillResult {
    pub bank_msg: BankMsg,
    pub event: Event,
}

pub type Denom = String;
pub type Pair = (Denom, Denom);
pub type SolverAddress = String;