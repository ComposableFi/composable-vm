//! AAA, collateral(stake)<->order, CVM route validation

use cosmwasm_std::{Addr, StdResult};

use crate::{OrderItem, SolvedOrder};

/// Validate solver can solve order he tells.
/// Minimal requirement is that CVM salt is unique to solver
pub fn validate_solver(
    _as_ref: cosmwasm_std::Deps<'_>,
    _sender: &Addr,
    _order: &OrderItem,
) -> StdResult<()> {
    Ok(())
}

/// Validate solver can solver amount he claimed
pub(crate) fn validate_solvers(
    _deps: &cosmwasm_std::DepsMut<'_>,
    _solution: &crate::SolutionItem,
    _all_orders: &[SolvedOrder],
) -> StdResult<()> {
    Ok(())
}

/// Validate solver program is sane
/// Minimal requirement is that CVM salt is unique to solver
pub(crate) fn validate_routes(
    _deps: &cosmwasm_std::DepsMut<'_>,
    _solution: &crate::SolutionItem,
    _all_orders: &[SolvedOrder],
) -> StdResult<()> {
    Ok(())
}
