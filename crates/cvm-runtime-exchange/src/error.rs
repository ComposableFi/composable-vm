use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("Only single asset exchange is supported by pool")]
    OnlySingleAssetExchangeIsSupportedByPool,

    /// for the case when specific pool does not supports slippage
    #[error("Exchange does not support slippage")]
    ExchangeDoesNotSupportSlippage,

    #[error("Cannot define both slippage and limit at same time")]
    CannotDefineBothSlippageAndLimitAtSameTime,

    #[error("Asset not found: {0}")]
    AssetNotFound(StdError),
    #[error("Exchange not found: {0}")]
    ExchangeNotFound(StdError),

    #[error("An error occured while doing arithmetic operations")]
    ArithmeticError,

    #[error("Program is invalid")]
    InvalidProgram,

    #[error("{0}")]
    Std(#[from] StdError),
}

impl From<cvm::ArithmeticError> for ContractError {
    fn from(_: cvm::ArithmeticError) -> Self {
        ContractError::InvalidProgram
    }
}
