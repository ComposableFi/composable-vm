//! given program gives route information as far as it can
use crate::prelude::*;
use cvm_runtime::shared::XcProgram;

pub(crate) fn get_route(
    deps: cosmwasm_std::Deps<'_>,
    program: XcProgram,
) -> Result<cvm_runtime::prelude::Binary, crate::error::ContractError> {
    todo!()
}
