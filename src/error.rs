use cosmwasm_std::{StdError, Uint256};
use thiserror::Error;

/// Custom errors that can be thrown from our contract.
#[derive(Error, Debug)]
pub enum ContractError {
    /// Wrap `StdError` for rethrowing cosmwasm errors.
    #[error("{0}")]
    Std(#[from] StdError),

    /// Attempted to create a pool which already exists.
    #[error("Pool already exists chain0_id - {chain0_id:?}, chain1_id - {chain1_id:?}, token0 - {token0:?}, token1 - {token1:?}")]
    PoolExists {
        /// The source chain.
        chain0_id: Uint256,
        /// The target chain.
        chain1_id: Uint256,
        /// The source chain token.
        token0: String,
        /// The target chain token.
        token1: String,
    },
}
