use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Timestamp, Uint256};
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct PoolInfo {
    pub pool_id: Uint256,
    pub meta: PoolMetaInfo,
    pub amount0: Uint256,
    pub amount1: Uint256,
    pub timestamp: Timestamp,
    pub chain0_init_depositor: String,
    pub chain1_init_depositor: String,
}

#[cw_serde]
pub struct PoolMetaInfo {
    pub chain0_id: Uint256,
    pub chain1_id: Uint256,
    pub token0: String,
    pub token1: String,
}

pub const POOLS_INFO: Map<&[u8], PoolInfo> = Map::new("pools_info");

pub const POOL_IDS: Map<&[u8], Uint256> = Map::new("pools_ids");

pub const POOLS_COUNT: Item<Uint256> = Item::new("pools_count");

pub const DEADLINE: Item<u64> = Item::new("deadline");

pub const LIQUIDITY: Map<&[u8], Uint256> = Map::new("liquidity");

pub const LIQUIDITY_QUEUE: Map<u64, Liquidity> = Map::new("liquidity_queue");
