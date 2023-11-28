use cosmwasm_std::{BankMsg, Event, StdResult, Uint64};
use cvm_runtime::{ExchangeId, NetworkId, shared::XcProgram};

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
    #[no_panic]
    pub fn fill(&mut self, wanted_transfer: Uint128) {
        // was given more or exact wanted - user happy or user was given all before, do not give more
        if wanted_transfer >= self.msg.wants.amount || self.msg.wants.amount.u128() == <_>::default() {
            self.given.amount = <_>::default();
            self.msg.wants.amount = <_>::default();
        } else {
            self.msg.wants.amount = self.msg.wants.amount.checked_sub(wanted_transfer).expect("proven above via comparison");
            let given_reduction = wanted_transfer * self.given.amount / self.msg.wants.amount;
                        
            self.given.amount = self.given.amount.saturating_sub(given_reduction);
        }
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
    pub owner: Addr,
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
    /// all CoWs ensured to be solved against one optimal price
    pub optimal_price: (u64, u64),
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
    pub route: XcProgram,
}

/// how much of order to be solved by CoW.
/// difference with `Fill` to be solved by cross chain exchange
/// aggregate pool of all orders in solution is used to give user amount he wants.
#[cw_serde]
pub struct Cow {
    pub order_id: OrderId,
    /// how much of order to be solved by from bank for all aggregated cows, `want` unit
    pub cow_amount: Amount,
    /// amount user should get after order executed in `want` unit
    pub given: Amount,
}
impl Cow {
    pub fn new(order_id: OrderId, cow_amount: Amount, given: Amount) -> Self {
        assert!(cow_amount <= given);
        Self {
            order_id,
            cow_amount,
            given,
        }
    }
}

#[cw_serde]
pub struct SolvedOrder {
    pub order: OrderItem,
    pub solution: Cow,
}

impl SolvedOrder {
    /// if given less, it will be partial, validated via bank
    /// if given more, it is over limit - user is happy, and total verified via bank
    pub fn new(order: OrderItem, solution: Cow) -> StdResult<Self> {
        Ok(Self { order, solution })
    }

    pub fn pair(&self) -> Pair {
        let mut pair = (self.order.given.denom.clone(), self.order.msg.wants.denom.clone());
        pair.sort_selection();
        pair
    } 

    pub fn cross_chain(&self) -> u128 {
        self.order.msg.wants.amount.u128() - self.solution.cow_amount.u128()
    }

    pub fn filled(&self) -> u128 {
        self.solution.cow_amount.u128()
    }

    pub fn wanted_denom(&self) -> String {
        self.order.msg.wants.denom.clone()
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

/// when solution is applied to order item,
/// what to ask from host to do next
pub struct CowFillResult {
    pub bank_msg: BankMsg,
    pub event: Event,
}

pub type Denom = String;
pub type Pair = (Denom, Denom);
pub type SolverAddress = String;
