//! Messages used to instantiate/execute/query the contract.

use crate::state::{ChainInfo, LiquidityQueueElement, PoolInfo, QueueID, State};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Binary, CustomMsg, Uint256};

/// Arguments to instantiate our contract.
#[cw_serde]
pub struct InstantiateMsg {
    /// Paloma address that will run transactions with target chain event information.
    pub event_tracker: Addr,
    /// Deadline for when the pool first has liquidity.
    pub deadline: u64,
}

/// Arguments to execute one of our subfunctions.
#[cw_serde]
pub enum ExecuteMsg {
    /// Register a new chain.
    RegisterChain {
        /// The chain ID defined by administrator(can be different than chainId of EVM chains).
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
    /// Update configurations.
    UpdateConfig {
        /// Interval before trades are considered invalid.
        new_deadline: Option<u64>,
        /// Management fee.
        new_fee: Option<u16>,
        /// Administrator.
        new_admin: Option<Addr>,
        /// Event tracker from target chains.
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
    /// Returns registered chain information.
    #[returns(ChainInfo)]
    ChainInfo {
        /// Chain Id to get chain information.
        chain_id: Uint256,
    },

    /// Returns pool_id number from chain and token information.
    #[returns(Uint256)]
    PoolId {
        /// Chain 0 Id of the pool.
        chain0_id: Uint256,
        /// Chain 1 Id of the pool.
        chain1_id: Uint256,
        /// Token address on chain 0.
        token0: String,
        /// Token address on chain 1.
        token1: String,
    },
    /// Returns pool information from pool_id.
    #[returns(PoolInfo)]
    PoolInfo {
        /// Pool id to get Pool information.
        pool_id: Uint256,
    },

    /// Returns state information.
    #[returns(State)]
    State {},

    /// Returns liquidity queue information.
    #[returns(QueueID)]
    LiquidityQueue {
        /// Pool id to get liquidity queue information.
        pool_id: Uint256,
    },

    /// Returns liquidity queue element data from pool_id and queue_id.
    #[returns(LiquidityQueueElement)]
    LiquidityQueueElement {
        /// Pool id to get liquidity queue element.
        pool_id: Uint256,
        /// The sequence number to get liquidity queue element of the pool.
        queue_id: u64,
    },
}

impl CustomMsg for PalomaMsg {}
