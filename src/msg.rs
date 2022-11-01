//! Messages used to instantiate/execute/query the contract.

use crate::state::{ChainInfo, LiquidityQueueElement, PoolInfo, QueueID, State};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Binary, CustomMsg, Uint256};

/// Arguments to instantiate our contract.
#[cw_serde]
pub struct InstantiateMsg {
    pub event_tracker: Addr,
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
        /// Chain name i.e. "Ethereum mainnet".
        chain_name: String,
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
        /// Swap fee amount.
        fee: u16,
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
    UpdateConfig {
        new_deadline: Option<u64>,
        new_fee: Option<u16>,
        new_admin: Option<Addr>,
        new_event_tracker: Option<Addr>,
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
pub enum QueryMsg {
    #[returns(ChainInfo)]
    ChainInfo { chain_id: Uint256 },

    #[returns(Uint256)]
    PoolId {
        chain0_id: Uint256,
        chain1_id: Uint256,
        token0: String,
        token1: String,
    },

    #[returns(PoolInfo)]
    PoolInfo { pool_id: Uint256 },

    #[returns(State)]
    State {},

    #[returns(QueueID)]
    LiquidityQueue { pool_id: Uint256 },

    #[returns(LiquidityQueueElement)]
    LiquidityQueueElement { pool_id: Uint256, queue_id: u64 },
}

impl CustomMsg for PalomaMsg {}
