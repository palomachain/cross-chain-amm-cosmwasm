use cosmwasm_std::{StdError, Uint256};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Pool already exists chain0_id - {chain0_id:?}, chain1_id - {chain1_id:?}, token0 - {token0:?}, token1 - {token1:?}")]
    PoolExists {
        chain0_id: Uint256,
        chain1_id: Uint256,
        token0: String,
        token1: String,
    },
    // Add any other custom errors you like here.
    // Look at https://docs.rs/thiserror/1.0.21/thiserror/ for details.
}
