#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint256};
// use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};

/*
// version info for migration info
const CONTRACT_NAME: &str = "crates.io:cross-chain-amm-cosmwasm";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
*/

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    unimplemented!()
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::CreatePool {
            chain0_id,
            chain1_id,
            token0,
            token1,
            chain0_init_depositor,
            chain1_init_depositor,
        } => create_pool(
            chain0_id,
            chain1_id,
            token0,
            token1,
            chain0_init_depositor,
            chain1_init_depositor,
        ),
        ExecuteMsg::Swap {
            chain_from_id,
            chain_to_id,
            token_from,
            token_to,
            sender,
            receiver,
            amount,
        } => swap(
            chain_from_id,
            chain_to_id,
            token_from,
            token_to,
            sender,
            receiver,
            amount,
        ),
        ExecuteMsg::AddLiquidity {
            chain_id,
            token,
            amount,
            sender,
            receiver,
        } => add_liquidity(chain_id, token, amount, sender, receiver),
        ExecuteMsg::RemoveLiquidity {
            chain0_id,
            chain1_id,
            token0,
            token1,
            receiver0,
            receiver1,
            amount,
        } => remove_liquidity(
            chain0_id, chain1_id, token0, token1, receiver0, receiver1, amount,
        ),
    }
}

fn create_pool(
    chain0_id: Uint256,
    chain1_id: Uint256,
    token0: String,
    token1: String,
    chain0_init_depositor: String,
    chain1_init_depositor: String,
) -> Result<Response, ContractError> {
    unimplemented!()
}

fn swap(
    chain_from_id: Uint256,
    chain_to_id: Uint256,
    token_from: String,
    token_to: String,
    sender: String,
    receiver: String,
    amount: Uint256,
) -> Result<Response, ContractError> {
    unimplemented!()
}

fn add_liquidity(
    chain_id: Uint256,
    token: String,
    amount: Uint256,
    sender: String,
    receiver: Addr,
) -> Result<Response, ContractError> {
    unimplemented!()
}

fn remove_liquidity(
    chain0_id: Uint256,
    chain1_id: Uint256,
    token0: String,
    token1: String,
    receiver0: String,
    receiver1: String,
    amount: Uint256,
) -> Result<Response, ContractError> {
    unimplemented!()
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, _msg: QueryMsg) -> StdResult<Binary> {
    unimplemented!()
}

#[cfg(test)]
mod tests {}
