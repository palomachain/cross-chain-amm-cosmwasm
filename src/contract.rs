use crate::ContractError::PoolExists;
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint256,
};
// use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{PoolInfo, PoolMetaInfo, DEADLINE, POOLS_COUNT, POOLS_INFO, POOL_IDS};

/*
// version info for migration info
const CONTRACT_NAME: &str = "crates.io:cross-chain-amm-cosmwasm";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
*/

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    POOLS_COUNT.save(deps.storage, &Uint256::zero())?;
    DEADLINE.save(deps.storage, &msg.deadline)?;
    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: ExecuteMsg,
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
            deps,
            env,
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
            pool_id,
            chain_id,
            token,
            amount,
            sender,
            receiver,
        } => add_liquidity(pool_id, chain_id, token, amount, sender, receiver),
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
    deps: DepsMut,
    env: Env,
    chain0_id: Uint256,
    chain1_id: Uint256,
    token0: String,
    token1: String,
    chain0_init_depositor: String,
    chain1_init_depositor: String,
) -> Result<Response, ContractError> {
    let pool_meta_info = PoolMetaInfo {
        chain0_id,
        chain1_id,
        token0: token0.clone(),
        token1: token1.clone(),
    };
    let bin = to_binary(&pool_meta_info)?;
    let meta_info_key = bin.as_slice();
    let mut pool_id;
    if POOL_IDS.has(deps.storage, meta_info_key) {
        let id = POOL_IDS.load(deps.storage, meta_info_key)?;
        let pool_info = POOLS_INFO.load(deps.storage, id.to_be_bytes().as_slice())?;
        if (pool_info.amount0.is_zero() || pool_info.amount1.is_zero())
            && pool_info
                .timestamp
                .plus_seconds(DEADLINE.load(deps.storage)?)
                < env.block.time
        {
            pool_id = id;
        } else {
            return Err(PoolExists {
                chain0_id,
                chain1_id,
                token0,
                token1,
            });
        }
    } else {
        pool_id = POOLS_COUNT.load(deps.storage)?;
    }
    let pool_info = PoolInfo {
        pool_id,
        meta: pool_meta_info,
        amount0: Uint256::zero(),
        amount1: Uint256::zero(),
        timestamp: env.block.time,
        chain0_init_depositor,
        chain1_init_depositor,
    };

    POOL_IDS.save(deps.storage, meta_info_key, &pool_id)?;
    POOLS_INFO.save(deps.storage, pool_id.to_be_bytes().as_slice(), &pool_info)?;
    Ok(Response::new())
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
    pool_id: Uint256,
    chain_id: Uint256,
    token: String,
    amount: Uint256,
    sender: String,
    receiver: Addr,
) -> Result<Response, ContractError> {
    let pool_info = POOLS_INFO.load(deps.storage, pool_id.to_be_bytes().as_slice())?;
    if pool_info.meta.chain0_id == chain_id {
        assert_eq!(pool_info.meta.token0, token);

    }
    Ok(Response::new())
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
