use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Binary, CustomMsg, Uint256};

#[cw_serde]
pub struct InstantiateMsg {
    pub target_contract_info: TargetContractInfo,
    pub deadline: u64, // Pool first add liquidity period
}

#[cw_serde]
pub enum ExecuteMsg {
    RegisterChain{
        chain_id: Uint256,
        factory: String,
    },
    CreatePool{
        chain0_id: Uint256,
        chain1_id: Uint256,
        token0: String,
        token1: String,
        chain0_init_depositor: String,
        chain1_init_depositor: String
    },
    Swap {
        chain_from_id: Uint256,
        chain_to_id: Uint256,
        token_from: String,
        token_to: String,
        sender: String,
        receiver: String,
        amount: Uint256,
    },
    AddLiquidity {
        pool_id: Uint256,
        chain_id: Uint256,
        token: String,
        amount: Uint256,
        sender: String,
        receiver: Addr
    },
    RemoveLiquidity {
        chain0_id: Uint256,
        chain1_id: Uint256,
        token0: String,
        token1: String,
        receiver0: String,
        receiver1: String,
        amount: Uint256,
    }
}

#[cw_serde]
pub struct TargetContractInfo {
    pub method: String,
    pub chain_id: String,
    pub compass_id: String,
    pub contract_address: String,
    pub smart_contract_abi: String,
}

#[cw_serde]
pub struct PalomaMsg {
    pub target_contract_info: TargetContractInfo,
    pub payload: Binary,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {}

impl CustomMsg for PalomaMsg {}
