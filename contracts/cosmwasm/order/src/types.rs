use cosmwasm_std::{ensure, BankMsg, Event, StdResult};
use cvm_runtime::{outpost::ExecuteProgramMsg, shared::CvmProgram, AssetId};
use mantis_cw::{DenomPair, OrderSide};

pub type Ratio = (u64, u64); // num_rational::Ratio<u64>;

use crate::prelude::*;

pub type OrderId = Uint128;

pub type Amount = Uint128;

/// block moment (analog of timestamp)
pub type Block = u64;

/// each CoW solver locally, is just transfer from shared pair bank with referenced order
pub type CowFilledOrder = (Coin, OrderId);

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct CowSolutionCalculation {
    pub token_a_remaining: Amount,
    pub token_b_remaining: Amount,
    pub filled: Vec<CowFilledOrder>,
}

#[cfg_attr(
    feature = "json-schema", // all(feature = "json-schema", not(target_arch = "wasm32")),
    derive(schemars::JsonSchema)
)]
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct OrderItem {
    pub owner: Addr,
    pub msg: OrderSubMsg,
    pub given: Coin,
    pub order_id: OrderId,
}

impl OrderItem {
    pub fn side(&self, pair: &DenomPair) -> OrderSide {
        if self.given.denom == pair.a {
            OrderSide::A
        } else {
            OrderSide::B
        }
    }

    pub fn pair(&self) -> DenomPair {
        DenomPair::new(self.given.denom.clone(), self.msg.wants.denom.clone())
    }

    /// `wanted_fill_amount` - amount to fill in `wants` amounts
    /// Reduces given amount
    /// `optimal_price` - the price to solve against, should be same or better than user limit.
    pub fn fill(&mut self, wanted_fill_amount: Amount, _optimal_ratio: Ratio) -> StdResult<()> {
        // was given more or exact wanted - user happy or user was given all before, do not give more
        if wanted_fill_amount >= self.msg.wants.amount
            || self.msg.wants.amount.u128() == <_>::default()
        {
            self.given.amount = <_>::default();
            self.msg.wants.amount = <_>::default();
        } else {
            let original_given = self.given.amount;
            let given_reduction = wanted_fill_amount
                .checked_mul(self.given.amount)?
                .checked_div(self.msg.wants.amount)?;

            self.msg.wants.amount = self.msg.wants.amount.checked_sub(wanted_fill_amount)?;
            self.given.amount = self.given.amount.saturating_sub(given_reduction);
            ensure!(
                self.given.amount < original_given,
                crate::errors::amount_does_not_decrease_want()
            );
            assert!(self.given.amount > <_>::default());
        }
        Ok(())
    }

    /// remaining after want fill using optimal price
    pub fn remaining(&self, wanted_fill_amount: Amount, optimal_ratio: Ratio) -> Amount {
        let mut slow = self.clone();
        slow.fill(wanted_fill_amount, optimal_ratio)
            .expect("off chain ok");
        self.given.amount - slow.given.amount
    }
}

/// simple structure which can be applied to order to fill or partial fill it
pub struct Filling {
    pub order_id: u64,
    pub amount: Uint128,
}

#[cfg(test)]
mod test {
    use cosmwasm_std::Coin;
    use num_rational::Ratio;

    use crate::prelude::*;
    use crate::types::*;

    #[test]
    pub fn fill() {
        let optimal_price = Ratio::new(1u64, 1u64);
        let mut order = OrderItem {
            owner: Addr::unchecked("owner".to_string()),
            msg: OrderSubMsg {
                wants: Coin {
                    denom: "wants".to_string(),
                    amount: 100u128.into(),
                },
                transfer: None,
                timeout: 1,
                min_fill: None,
            },
            given: Coin {
                denom: "given".to_string(),
                amount: 100u128.into(),
            },
            order_id: 1u128.into(),
        };
        order.fill(50u128.into(), optimal_price).unwrap();
        assert_eq!(order.given.amount, Uint128::from(50u128));
        assert_eq!(order.msg.wants.amount, Uint128::from(50u128));
        order.fill(15u128.into(), optimal_price).unwrap();
        assert_eq!(order.given.amount, Uint128::from(35u128));
        assert_eq!(order.msg.wants.amount, Uint128::from(35u128));
        order.fill(Uint128::from(50u128), optimal_price).unwrap();
        assert_eq!(order.given.amount, Uint128::from(0u128));
        assert_eq!(order.msg.wants.amount, Uint128::from(0u128));

        let mut order = OrderItem {
            owner: Addr::unchecked("owner".to_string()),
            msg: OrderSubMsg {
                wants: Coin {
                    denom: "wants".to_string(),
                    amount: 2000000000u128.into(),
                },
                transfer: None,
                timeout: 1,
                min_fill: None,
            },
            given: Coin {
                denom: "given".to_string(),
                amount: 100u128.into(),
            },
            order_id: 1u128.into(),
        };

        assert!(order.fill(500u128.into(), optimal_price).is_err());
        order.fill(50000000u128.into(), optimal_price).unwrap();
        assert_eq!(order.given.amount, Uint128::from(98u128));
    }
}

#[cfg_attr(
    feature = "json-schema", // all(feature = "json-schema", not(target_arch = "wasm32")),
    derive(schemars::JsonSchema)
)]
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
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
    pub transfer: Option<AssetId>,
    /// until what block to wait for solution, if none, then cleaned up
    pub timeout: Block,
    /// if ok with partial fill, what is the minimum amount
    pub min_fill: Option<Ratio>,
}

#[cw_serde]
pub struct SolutionItem {
    pub pair: DenomPair,
    pub msg: SolutionSubMsg,
    /// at which block solution was added
    pub block_added: u64,
    pub owner: Addr,
}

impl SolutionItem {
    pub fn id(&self) -> Vec<u8> {
        solution_id(&(self.owner.to_string(), self.pair.clone(), self.block_added))
    }
}

#[cfg_attr(
    feature = "json-schema", // all(feature = "json-schema", not(target_arch = "wasm32")),
    derive(schemars::JsonSchema)
)]
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct CrossChainPart {
    pub msg: ExecuteProgramMsg,
    /// what price is used to take from orders
    pub optimal_price: Ratio,
}

impl CrossChainPart {
    pub fn new(program: CvmProgram, salt: Vec<u8>, optimal_price: Ratio) -> Self {
        Self {
            msg: ExecuteProgramMsg {
                program,
                salt,
                assets: None,
                tip: None,
            },
            optimal_price,
        }
    }
}

/// price information will not be used on chain or deciding.
/// it will fill orders on chain as instructed
/// and check that max/min from orders respected
/// and sum all into volume. and compare solutions.
/// on chain cares each user gets what it wants and largest volume solution selected.
#[cw_serde]
pub struct SolutionSubMsg {
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub cows: Vec<OrderSolution>,
    /// all CoWs ensured to be solved against one optimal price
    pub optimal_price: Ratio,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub route: Option<CrossChainPart>,

    /// after some time, solver will not commit to success
    pub timeout: Block,
}

/// after cows solved, need to route remaining cross chain
#[cfg_attr(
    feature = "json-schema", // all(feature = "json-schema", not(target_arch = "wasm32")),
    derive(schemars::JsonSchema)
)]
#[derive(Clone, Debug, Serialize, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct RouteSubMsg {
    pub solved_orders: Vec<SolvedOrder>,
    pub msg: CrossChainPart,
    pub solution_id: SolutionHash,
    pub solver_address: String,
    pub pair: DenomPair,
}

#[cfg_attr(
    feature = "json-schema", // all(feature = "json-schema", not(target_arch = "wasm32")),
    derive(schemars::JsonSchema)
)]
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SubWasmMsg<Payload> {
    pub msg: Payload,
    pub funds: Vec<Coin>,
}

#[cfg_attr(
    feature = "json-schema", // all(feature = "json-schema", not(target_arch = "wasm32")),
    derive(schemars::JsonSchema)
)]
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Copy)]
#[serde(rename_all = "snake_case")]
pub enum OrderAmount {
    /// whole remaining amount in order
    /// (same as part, when calculated manually)
    /// In case of zero will not cross chain it.
    AllRemaining,
    /// In and out amount. Must be above optimal price.
    /// .0 how much to take from user for cross chain routing, in `given` unit
    /// .1 how much to dispatch to user after routing, in `wants` unit
    Part(Amount, Amount),
}

/// how much of order to be solved by CoW.
/// difference with `Fill` to be solved by cross chain exchange
/// aggregate pool of all orders in solution is used to give user amount he wants.
#[cfg_attr(
    feature = "json-schema", // all(feature = "json-schema", not(target_arch = "wasm32")),
    derive(schemars::JsonSchema)
)]
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct OrderSolution {
    pub order_id: OrderId,
    /// how much of order to be solved by from bank for this order, `want` unit
    pub cow_out_amount: Amount,
    pub cross_chain_part: Option<OrderAmount>,
}
impl OrderSolution {
    pub fn new_part_local_remaining_remote(order_id: OrderId, cow_out_amount: Amount) -> Self {
        Self {
            order_id,
            cow_out_amount,
            cross_chain_part: Some(OrderAmount::AllRemaining),
        }
    }
}

/// We have current status of order, and current solution found by solver.
#[cfg_attr(
    feature = "json-schema", // all(feature = "json-schema", not(target_arch = "wasm32")),
    derive(schemars::JsonSchema)
)]
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SolvedOrder {
    pub order: OrderItem,
    pub solution: OrderSolution,
}
impl SolvedOrder {
    pub fn given_cross_chain(&self) -> Amount {
        match self.solution.cross_chain_part {
            Some(x) => match x {
                OrderAmount::AllRemaining => self.order.given.amount,
                OrderAmount::Part(x, _) => x,
            },
            None => 0u128.into(),
        }
    }

    pub fn wants_cross_chain(&self) -> Amount {
        match self.solution.cross_chain_part {
            Some(x) => match x {
                OrderAmount::AllRemaining => self.order.msg.wants.amount,
                OrderAmount::Part(_, x) => x,
            },
            None => 0u128.into(),
        }
    }
}

#[cfg_attr(
    feature = "json-schema", // all(feature = "json-schema", not(target_arch = "wasm32")),
    derive(schemars::JsonSchema)
)]
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]

pub struct TrackedOrderItem {
    pub order_id: OrderId,
    pub solution_id: SolutionHash,
    pub amount_taken: Coin,
    pub promised: Amount,
}

impl SolvedOrder {
    /// if given less, it will be partial, validated via bank
    /// if given more, it is over limit - user is happy, and total verified via bank
    pub fn new(order: OrderItem, solution: OrderSolution) -> StdResult<Self> {
        Ok(Self { order, solution })
    }

    pub fn pair(&self) -> DenomPair {
        DenomPair::new(
            self.order.given.denom.clone(),
            self.order.msg.wants.denom.clone(),
        )
    }

    pub fn cross_chain(&self) -> u128 {
        self.order.msg.wants.amount.u128() - self.solution.cow_out_amount.u128()
    }

    pub fn filled(&self) -> u128 {
        self.solution.cow_out_amount.u128()
    }

    pub fn wanted_denom(&self) -> String {
        self.order.msg.wants.denom.clone()
    }

    pub fn other(&self, denom: &str) -> String {
        if self.order.given.denom == denom {
            self.order.msg.wants.denom.clone()
        } else {
            self.order.given.denom.clone()
        }
    }

    pub fn given_denom(&self) -> String {
        self.order.given.denom.clone()
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
    pub remaining: Option<OrderItem>,
    pub bank_msg: BankMsg,
    pub event: Event,
}

pub struct CvmFillResult {
    pub tracking: TrackedOrderItem,
    pub remaining: Option<OrderItem>,
    pub event: Event,
}

impl CvmFillResult {
    pub fn new(tracking: TrackedOrderItem, event: Event) -> Self {
        Self {
            tracking,
            remaining: None,
            event,
        }
    }
}

pub type SolverAddress = String;

pub type CrossChainSolutionKey = (SolverAddress, DenomPair, Block);

pub type SolutionHash = Vec<u8>;

pub fn solution_id(id: &CrossChainSolutionKey) -> SolutionHash {
    use sha2::*;
    let mut hash = Sha256::new();
    hash.update(id.0.as_bytes());
    hash.update(id.1.a.as_bytes());
    hash.update(id.1.b.as_bytes());
    hash.update(id.2.to_be_bytes());
    hash.finalize().to_vec()
}
