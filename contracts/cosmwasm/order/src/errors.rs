use cosmwasm_std::StdError;

pub fn amount_does_not_decrease_want() -> StdError {
    StdError::generic_err("Amount does not decrease want")
}

pub fn expected_some_funds_in_order() -> StdError {
    StdError::generic_err("Expected some funds in order")
}
