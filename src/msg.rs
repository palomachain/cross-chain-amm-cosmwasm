//! Messages used to instantiate/execute/query the contract.

use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Binary, CustomMsg, Uint256};

/// Arguments to instantiate our contract.
#[cw_serde]
pub struct InstantiateMsg {
    /// Deadline for when the pool first has liquidity.
    pub deadline: u64,
}

/// Arguments to execute one of our subfunctions.
#[cw_serde]
pub enum ExecuteMsg {
    /// Register a new chain.
    RegisterChain {
        /// The chain ID.
        chain_id: Uint256,
        /// The factory contract we will use to mint tokens.
        factory: String,
    },
    /// Instantiate a new pool.
    CreatePool {
        /// Source chain id.
        chain0_id: Uint256,
        /// Target chain id.
        chain1_id: Uint256,
        /// Source chain token.
        token0: String,
        /// Target chain token.
        token1: String,
        /// Source chain depositor.
        chain0_init_depositor: String,
        /// Target chain depositor.
        chain1_init_depositor: String,
    },
    /// Initiate a swap.
    Swap {
        /// Source chain id.
        chain_from_id: Uint256,
        /// Target chain id.
        chain_to_id: Uint256,
        /// Source chain token.
        token_from: String,
        /// Target chain token.
        token_to: String,
        /// Source account.
        sender: String,
        /// Target account.
        receiver: String,
        /// Amount to transfer.
        amount: Uint256,
    },
    /// Add funds to a pool.
    AddLiquidity {
        /// Pool to add liquidity to.
        pool_id: Uint256,
        /// Chain to add liquidity to.
        chain_id: Uint256,
        /// Token to add liquidity in.
        token: String,
        /// Amount we are adding.
        amount: Uint256,
        /// Address sending funds.
        sender: String,
        /// Address receiving funds.
        receiver: Addr,
    },
    /// Transfer funds between cross chain accounts.
    RemoveLiquidity {
        /// Chain to transfer from.
        chain0_id: Uint256,
        /// Chain to transfer to.
        chain1_id: Uint256,
        /// Token to transfor from.
        token0: String,
        /// Token to transfer to.
        token1: String,
        /// Receiver0 address.
        receiver0: String,
        /// Receiver1 address.
        receiver1: String,
        /// Amount to transfer.
        amount: Uint256,
    },
}

/// Message struct for cross-chain calls.
#[cw_serde]
pub struct PalomaMsg {
    /// The ID of the paloma scheduled job to run.
    pub job_id: String,
    /// The payload, ABI encoded for the target chain.
    pub payload: Binary,
}

/// Currently cc-amm provides no queries.
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {}

impl CustomMsg for PalomaMsg {}
