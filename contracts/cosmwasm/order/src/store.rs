use cw_storage_plus::MultiIndex;

use crate::*;

/// so we need to have several solution per pair to pick one best
pub struct SolutionIndexes<'a> {
    /// (token pair secondary index), (stored item), (stored item full key)
    pub pair: MultiIndex<'a, Pair, SolutionItem, (Denom, Denom, SolverAddress)>,
}

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
