use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("InstantiateFailed")]
    InstantiateFailed {},

    #[error("TransferFailed(Not enough native sent)")]
    TransferFailed {},

    #[error("health factor is below MIN")]
    HealthFactorLess {},

    #[error("health factor is in safe range")]
    HealthFactorSafe {},

    #[error("Custom Error val: {val:?}")]
    CustomError { val: String },

    #[error("Uknown reply id: {id}")]
    UnknownReplyId { id: u64 },
}
