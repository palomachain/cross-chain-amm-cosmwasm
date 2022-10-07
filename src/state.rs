use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Timestamp, Uint256};
use cw_storage_plus::Map;

#[cw_serde]
pub struct PoolInfo {
    pub pool_id: Uint256,
    pub meta: PoolMetaInfo,
    pub amount0: Uint256,
    pub amount1: Uint256,
    pub timestamp: Timestamp,
}

#[cw_serde]
pub struct PoolMetaInfo {
    pub chain0_id: Uint256,
    pub chain1_id: Uint256,
    pub token0: String,
    pub token1: String,
}

pub const POOLS_INFO: Map<Uint256, PoolInfo> = Map::new("pools_info");

pub const POOLS_IDS: Map<PoolMetaInfo, Uint256> = Map::new("pools_ids");
