//! The persistent state of the contract, including pool info and associated queues.

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Timestamp, Uint256};
use cw_storage_plus::{Item, Map};

/// Metadata definiting a pool.
#[cw_serde]
pub struct PoolInfo {
    /// The ID for this pool.
    pub pool_id: Uint256,
    /// Chain/token pairs to trade.
    pub meta: PoolMetaInfo,
    /// Source amount.
    pub amount0: Uint256,
    /// Target amount.
    pub amount1: Uint256,
    /// Amount remaining to be transferred from the source.
    pub pending_amount0: Uint256,
    /// Amount remaining to be transferred from the target.
    pub pending_amount1: Uint256,
    /// Total liquidity available in this pool.
    pub total_liquidity: Uint256,
    /// Creation time of this `PoolInfo`.
    pub timestamp: Timestamp,
    /// Initial creator of `chain0`.
    pub chain0_init_depositor: String,
    /// Initial creator of `chain1`.
    pub chain1_init_depositor: String,
}

/// The chain/token pair which defines a pool.
#[cw_serde]
pub struct PoolMetaInfo {
    /// Source chain of the pair.
    pub chain0_id: Uint256,
    /// Target chain of the pair.
    pub chain1_id: Uint256,
    /// Source chain token.
    pub token0: String,
    /// Target chain token.
    pub token1: String,
}

/// Mapping from `chain_id` to factory contract `job_id`.
pub const POOL_FACTORIES: Map<&[u8], String> = Map::new("pool_factories");

/// Mapping from `chain_id` to information about its creation.
pub const POOLS_INFO: Map<&[u8], PoolInfo> = Map::new("pools_info");

/// Mapping from meta info key to pool id.
pub const POOL_IDS: Map<&[u8], Uint256> = Map::new("pools_ids");

/// Current numbxer of pools.
pub const POOLS_COUNT: Item<Uint256> = Item::new("pools_count");

/// Interval before trades are considered invalid.
pub const DEADLINE: Item<u64> = Item::new("deadline");

/// Mapping from `(pool_id, receiver)` to an amount.
pub const LIQUIDITY: Map<(&[u8], &[u8]), Uint256> = Map::new("liquidity");

/// Metadata allowing use of a map as a queue.
#[cw_serde]
pub struct QueueID {
    /// Index of head of queue.
    pub start: u64,
    /// Length of queue.
    pub length: u64,
}

/// The `(chain_id, amount, reciever)` triple being processed in the liquidity pool.
#[cw_serde]
pub struct LiquidityQueueElement {
    /// Transferring chain id.
    pub chain_id: Uint256,
    /// Amount to transfer.
    pub amount: Uint256,
    /// Receiving address.
    pub receiver: Addr,
}

/// Mapping from pool id to queue metadata, which can be used to index into `LIQUIDITY_QUEUE`.
pub const LIQUIDITY_QUEUE_IDS: Map<&[u8], QueueID> = Map::new("liquidity_queue_ids");

/// A map of (pool_id, queue_id) to (chain_id, amount, depositor).
pub const LIQUIDITY_QUEUE: Map<(&[u8], &[u8]), LiquidityQueueElement> = Map::new("liquidity_queue");
