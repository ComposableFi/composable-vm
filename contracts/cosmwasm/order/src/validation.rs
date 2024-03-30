//! AAA, collateral(stake)<->order, CVM route validation

use cosmwasm_std::{Addr, StdResult};

use crate::{OrderItem, SolvedOrder};

/// Validate solver can solve order he tells.
/// Minimal requirement is that CVM salt is unique to solver
pub fn validate_solver(
    as_ref: cosmwasm_std::Deps<'_>,
    sender: &Addr,
    order: &OrderItem,
) -> StdResult<()> {
    Ok(())
}

/// Validate solver can solver amount he claimed
pub(crate) fn validate_solvers(
    deps: &cosmwasm_std::DepsMut<'_>,
    solution: &crate::SolutionItem,
    all_orders: &[SolvedOrder],
) -> StdResult<()> {
    Ok(())
}

/// Validate solver program is sane
/// Minimal requirement is that CVM salt is unique to solver
pub(crate) fn validate_routes(
    deps: &cosmwasm_std::DepsMut<'_>,
    solution: &crate::SolutionItem,
    all_orders: &[SolvedOrder],
) -> StdResult<()> {
    Ok(())
}
