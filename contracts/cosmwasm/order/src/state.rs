//! simple operation without constraint checks and calculations
use cw_storage_plus::MultiIndex;

use crate::*;



/// the connection description from first network to second
// pub(crate) const NETWORK_TO_NETWORK: Map<(NetworkId, NetworkId), NetworkToNetworkItem> =
//     Map::new("network_to_network");



// pub const BEST_A_PROIE: Ratio = ;

/// so we need to have several solution per pair to pick one best
pub struct SolutionIndexes<'a> {
    /// (token pair secondary index), (stored item), (stored item full key)
    pub pair: MultiIndex<'a, DenomPair, SolutionItem, (DenomPair, SolverAddress)>,
}

/// (DenomPair,SolverAddress) -> SolutionItem
pub type SolutionMultiMap<'a> =
    IndexedMap<'a, &'a (DenomPair, SolverAddress), SolutionItem, SolutionIndexes<'a>>;

impl<'a> IndexList<SolutionItem> for SolutionIndexes<'a> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<SolutionItem>> + '_> {
        let v: Vec<&dyn Index<SolutionItem>> = vec![&self.pair];
        Box::new(v.into_iter())
    }
}

pub fn solutions<'a>() -> SolutionMultiMap<'a> {
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
