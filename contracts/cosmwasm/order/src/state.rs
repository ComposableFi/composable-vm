//! simple operation without constraint checks and calculations
use cw_storage_plus::MultiIndex;

use crate::*;

/// so we need to have several solution per pair to pick one best
pub struct SolutionIndexes<'a> {
    /// (token pair secondary index), (stored item), (stored item full key)
    pub pair: MultiIndex<'a, Pair, SolutionItem, (Denom, Denom, SolverAddress)>,
}

/// (a,b,solver)
pub type SolutionMultiMap<'a> =
    IndexedMap<'a, &'a (Denom, Denom, SolverAddress), SolutionItem, SolutionIndexes<'a>>;

impl<'a> IndexList<SolutionItem> for SolutionIndexes<'a> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<SolutionItem>> + '_> {
        let v: Vec<&dyn Index<SolutionItem>> = vec![&self.pair];
        Box::new(v.into_iter())
    }
}

pub fn solutions<'a>(
) -> IndexedMap<'a, &'a (String, String, String), SolutionItem, SolutionIndexes<'a>> {
    let indexes = SolutionIndexes {
        pair: MultiIndex::new(
            |_pk: &[u8], d: &SolutionItem| d.pair.clone(),
            "pair_solver_address",
            "pair",
        ),
    };
    IndexedMap::new("solutions", indexes)
}

pub fn join_solution_with_orders(
    orders: &Map<'_, u128, OrderItem>,
    msg: &SolutionSubMsg,
    ctx: &ExecCtx<'_>,
) -> Result<Vec<SolvedOrder>, StdError> {
    let all_orders = msg
        .cows
        .iter()
        .map(|x| {
            orders
                .load(ctx.deps.storage, x.order_id.u128())
                .map_err(|_| StdError::not_found("order"))
                .and_then(|order| SolvedOrder::new(order, x.clone()))
        })
        .collect::<Result<Vec<SolvedOrder>, _>>()?;
    Ok(all_orders)
}

pub fn get_solutions(
    solutions: &SolutionMultiMap,
    storage: &dyn Storage,
) -> Result<Vec<SolutionItem>, StdError> {
    solutions
        .idx
        .pair
        .range(storage, None, None, Order::Ascending)
        .map(|r| r.map(|(_, x)| x))
        .collect()
}
