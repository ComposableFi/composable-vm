use cosmwasm_std::Uint128;

#[derive(Debug, displaydoc::Display)]
pub enum MantisError {
    /// `{order_id}` Matching order not found
    MatchingOrderNotFound { order_id: Uint128 },
    /// `{order_id}` Cow fill badly found because `{reason}`
    CowFillBadlyFound { order_id: Uint128, reason: String },
    /// Blackbox error: `{reason}`
    BlackboxError { reason: String },
}

impl std::error::Error for MantisError {}
