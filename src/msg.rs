use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Uint256};

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {
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
#[derive(QueryResponses)]
pub enum QueryMsg {}
