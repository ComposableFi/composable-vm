use cosmwasm_std::StdError;

pub fn amount_does_not_decrease_want() -> StdError {
    StdError::generic_err("Amount does not decrease want")
}

pub fn expected_some_funds_in_order() -> StdError {
    StdError::generic_err("Expected some funds in order")
}

pub fn filled_order_cannot_be_cross_chain_routed() -> StdError {
    StdError::generic_err("Filled order cannot be cross chain routed")
}

pub fn partial_cross_chain_not_implemented() -> StdError {
    StdError::generic_err("Partial cross chain not implemented")
}
