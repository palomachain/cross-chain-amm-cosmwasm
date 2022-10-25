//! Execute cross chain transactions.

use crate::ContractError::PoolExists;
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Addr, Binary, CosmosMsg, Deps, DepsMut, Env, Isqrt, MessageInfo, Response,
    StdResult, Uint256, Uint512,
};
use ethabi::{Address, Contract, Function, Param, ParamType, StateMutability, Token, Uint};
use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::str::FromStr;
// use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, PalomaMsg, QueryMsg};
use crate::state::{
    LiquidityQueueElement, PoolInfo, PoolMetaInfo, QueueID, DEADLINE, LIQUIDITY, LIQUIDITY_QUEUE,
    LIQUIDITY_QUEUE_IDS, POOLS_COUNT, POOLS_INFO, POOL_FACTORIES, POOL_IDS, TARGET_CONTRACT_INFO,
};

const MIN_LIQUIDITY: u16 = 1000u16;

/// Instantiate the contract. Initialize the pools.
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    TARGET_CONTRACT_INFO.save(deps.storage, &msg.target_contract_info)?;
    POOLS_COUNT.save(deps.storage, &Uint256::zero())?;
    DEADLINE.save(deps.storage, &msg.deadline)?;
    Ok(Response::new())
}

/// Execute the contract. See ExecuteMsg submessages for details.
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response<PalomaMsg>, ContractError> {
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
        } => add_liquidity(deps, pool_id, chain_id, token, amount, sender, receiver),
        ExecuteMsg::RemoveLiquidity {
            chain0_id,
            chain1_id,
            token0,
            token1,
            receiver0,
            receiver1,
            amount,
        } => remove_liquidity(
            deps, info, chain0_id, chain1_id, token0, token1, receiver0, receiver1, amount,
        ),
        ExecuteMsg::RegisterChain { chain_id, factory } => register_chain(deps, chain_id, factory),
    }
}

fn register_chain(
    deps: DepsMut,
    chain_id: Uint256,
    factory: String,
) -> Result<Response<PalomaMsg>, ContractError> {
    let binding = chain_id.to_be_bytes();
    let chain_id_key = binding.as_slice();
    assert!(!POOL_FACTORIES.has(deps.storage, chain_id_key));
    POOL_FACTORIES.save(deps.storage, chain_id_key, &factory)?;
    Ok(Response::new())
}

#[allow(clippy::too_many_arguments)]
fn create_pool(
    deps: DepsMut,
    env: Env,
    chain0_id: Uint256,
    chain1_id: Uint256,
    token0: String,
    token1: String,
    chain0_init_depositor: String,
    chain1_init_depositor: String,
) -> Result<Response<PalomaMsg>, ContractError> {
    assert!(chain0_id < chain1_id);

    let pool_meta_info = PoolMetaInfo {
        chain0_id,
        chain1_id,
        token0: token0.clone(),
        token1: token1.clone(),
    };
    let binding = to_binary(&pool_meta_info)?;
    let meta_info_key = binding.as_slice();
    let pool_id;
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
        pending_amount0: Uint256::zero(),
        pending_amount1: Uint256::zero(),
        total_liquidity: Uint256::zero(),
        timestamp: env.block.time,
        chain0_init_depositor,
        chain1_init_depositor,
    };

    POOL_IDS.save(deps.storage, meta_info_key, &pool_id)?;
    let binding = pool_id.to_be_bytes();
    let pool_id_key = binding.as_slice();
    POOLS_INFO.save(deps.storage, pool_id_key, &pool_info)?;
    LIQUIDITY_QUEUE_IDS.save(
        deps.storage,
        pool_id_key,
        &QueueID {
            start: 0,
            length: 0,
        },
    )?;
    let binding = chain0_id.to_be_bytes();
    let chain_id_key = binding.as_slice();
    let factory = POOL_FACTORIES.load(deps.storage, chain_id_key)?;
    #[allow(deprecated)]
    let contract = Contract {
        constructor: None,
        functions: BTreeMap::from_iter(vec![(
            "create_pool".to_string(),
            vec![Function {
                name: "create_pool".to_string(),
                inputs: vec![
                    Param {
                        name: "token".to_string(),
                        kind: ParamType::Address,
                        internal_type: None,
                    },
                    Param {
                        name: "pool_id".to_string(),
                        kind: ParamType::Uint(256),
                        internal_type: None,
                    },
                ],
                outputs: Vec::new(),
                constant: None,
                state_mutability: StateMutability::NonPayable,
            }],
        )]),
        events: BTreeMap::new(),
        errors: BTreeMap::new(),
        receive: false,
        fallback: false,
    };

    let mut target_contract_info = TARGET_CONTRACT_INFO.load(deps.storage)?;
    target_contract_info.contract_address = factory;
    target_contract_info.chain_id = chain0_id.to_string();
    let msg0 = CosmosMsg::Custom(PalomaMsg {
        target_contract_info: target_contract_info.clone(),
        payload: Binary(
            contract
                .function("create_pool")
                .unwrap()
                .encode_input(&[
                    Token::Address(Address::from_str(token0.as_str()).unwrap()),
                    Token::Uint(Uint::from_str(pool_id.to_string().as_str()).unwrap()),
                ])
                .unwrap(),
        ),
    });
    let binding = chain1_id.to_be_bytes();
    let chain_id_key = binding.as_slice();
    let factory = POOL_FACTORIES.load(deps.storage, chain_id_key)?;
    target_contract_info.contract_address = factory;
    target_contract_info.chain_id = chain1_id.to_string();
    let msg1 = CosmosMsg::Custom(PalomaMsg {
        target_contract_info,
        payload: Binary(
            contract
                .function("create_pool")
                .unwrap()
                .encode_input(&[
                    Token::Address(Address::from_str(token1.as_str()).unwrap()),
                    Token::Uint(Uint::from_str(pool_id.to_string().as_str()).unwrap()),
                ])
                .unwrap(),
        ),
    });
    let response = Response::new().add_message(msg0).add_message(msg1);
    Ok(response)
}

fn swap(
    _chain_from_id: Uint256,
    _chain_to_id: Uint256,
    _token_from: String,
    _token_to: String,
    _sender: String,
    _receiver: String,
    _amount: Uint256,
) -> Result<Response<PalomaMsg>, ContractError> {
    unimplemented!()
}

fn add_liquidity(
    deps: DepsMut,
    pool_id: Uint256,
    chain_id: Uint256,
    token: String,
    amount: Uint256,
    sender: String,
    receiver: Addr,
) -> Result<Response<PalomaMsg>, ContractError> {
    let binding = pool_id.to_be_bytes();
    let pool_id_key = binding.as_slice();
    let mut pool_info = POOLS_INFO.load(deps.storage, pool_id_key)?;
    assert!(!amount.is_zero());
    let is_chain0 = if pool_info.meta.chain0_id == chain_id && pool_info.meta.token0 == token {
        true
    } else {
        assert!(pool_info.meta.chain1_id == chain_id && pool_info.meta.token1 == token);
        false
    };

    if pool_info.total_liquidity.is_zero() {
        if is_chain0 {
            assert_eq!(sender, pool_info.chain0_init_depositor);
        } else {
            assert_eq!(sender, pool_info.chain1_init_depositor);
        }
    }

    let mut liquidity_queue_id = LIQUIDITY_QUEUE_IDS.load(deps.storage, pool_id_key)?;
    if liquidity_queue_id.length > 0 {
        let mut id = liquidity_queue_id.start;
        let liquidity_queue =
            LIQUIDITY_QUEUE.load(deps.storage, (pool_id_key, id.to_be_bytes().as_slice()))?;
        if liquidity_queue.chain_id == chain_id {
            LIQUIDITY_QUEUE.save(
                deps.storage,
                (pool_id_key, (id + 1).to_be_bytes().as_slice()),
                &LiquidityQueueElement {
                    chain_id,
                    amount,
                    receiver,
                },
            )?;
            liquidity_queue_id.length += 1;
            LIQUIDITY_QUEUE_IDS.save(deps.storage, pool_id_key, &liquidity_queue_id)?;
            if is_chain0 {
                pool_info.pending_amount0 += amount;
            } else {
                pool_info.pending_amount1 += amount;
            }
        } else if pool_info.total_liquidity.is_zero() {
            let liquidity = if pool_info.pending_amount0.is_zero() {
                pool_info.amount0 = amount;
                pool_info.amount1 = pool_info.pending_amount1;
                (pool_info.pending_amount1 * amount).isqrt()
            } else {
                pool_info.amount1 = amount;
                pool_info.amount0 = pool_info.pending_amount0;
                (pool_info.pending_amount0 * amount).isqrt()
            };
            if receiver.eq(&liquidity_queue.receiver) {
                LIQUIDITY.save(
                    deps.storage,
                    (pool_id_key, receiver.as_bytes()),
                    &(liquidity - Uint256::from(MIN_LIQUIDITY)),
                )?;
            } else {
                LIQUIDITY.save(
                    deps.storage,
                    (pool_id_key, receiver.as_bytes()),
                    &((liquidity - Uint256::from(MIN_LIQUIDITY)) / Uint256::from(2u8)),
                )?;
                LIQUIDITY.save(
                    deps.storage,
                    (pool_id_key, liquidity_queue.receiver.as_bytes()),
                    &((liquidity - Uint256::from(MIN_LIQUIDITY)) / Uint256::from(2u8)),
                )?;
            }
            pool_info.pending_amount0 = Uint256::zero();
            pool_info.pending_amount1 = Uint256::zero();
            pool_info.total_liquidity = liquidity;
            LIQUIDITY_QUEUE_IDS.save(
                deps.storage,
                pool_id_key,
                &QueueID {
                    start: 0,
                    length: 0,
                },
            )?;
        } else {
            let mut queue_amount = if is_chain0 {
                amount * pool_info.amount1 / pool_info.amount0
            } else {
                amount * pool_info.amount0 / pool_info.amount1
            };
            let mut input_amount = amount;
            let limit = liquidity_queue_id.start + liquidity_queue_id.length;
            while id < limit && !input_amount.is_zero() {
                let binding = id.to_be_bytes();
                let id_key = binding.as_slice();
                let mut liquidity_queue =
                    LIQUIDITY_QUEUE.load(deps.storage, (pool_id_key, id_key))?;
                let input_token = match liquidity_queue.amount.cmp(&queue_amount) {
                    Ordering::Less => {
                        queue_amount -= liquidity_queue.amount;
                        id += 1;
                        let new_amount = if is_chain0 {
                            pool_info.pending_amount1 -= liquidity_queue.amount;
                            Uint256::try_from(
                                Uint512::from(liquidity_queue.amount)
                                    * Uint512::from(pool_info.amount0)
                                    / Uint512::from(pool_info.amount1),
                            )
                            .unwrap()
                        } else {
                            pool_info.pending_amount0 -= liquidity_queue.amount;
                            Uint256::try_from(
                                Uint512::from(liquidity_queue.amount)
                                    * Uint512::from(pool_info.amount1)
                                    / Uint512::from(pool_info.amount0),
                            )
                            .unwrap()
                        };
                        if input_amount > new_amount {
                            input_amount -= new_amount;
                        } else {
                            queue_amount = Uint256::zero();
                            input_amount = Uint256::zero();
                        }
                        liquidity_queue.amount
                    }
                    Ordering::Equal => {
                        liquidity_queue_id.length -= id + 1 - liquidity_queue_id.start;
                        liquidity_queue_id.start = id + 1;
                        if is_chain0 {
                            pool_info.pending_amount1 -= queue_amount;
                        } else {
                            pool_info.pending_amount0 -= queue_amount;
                        };
                        queue_amount = Uint256::zero();
                        input_amount = Uint256::zero();
                        queue_amount
                    }
                    Ordering::Greater => {
                        liquidity_queue.amount -= queue_amount;
                        liquidity_queue_id.length -= id - liquidity_queue_id.start;
                        liquidity_queue_id.start = id;
                        if is_chain0 {
                            pool_info.pending_amount1 -= queue_amount;
                        } else {
                            pool_info.pending_amount0 -= queue_amount;
                        };
                        LIQUIDITY_QUEUE.save(
                            deps.storage,
                            (pool_id_key, id_key),
                            &liquidity_queue,
                        )?;
                        queue_amount = Uint256::zero();
                        input_amount = Uint256::zero();
                        queue_amount
                    }
                };

                let liq = if is_chain0 {
                    Uint256::try_from(
                        Uint512::from(input_token) * Uint512::from(pool_info.amount0)
                            / Uint512::from(pool_info.amount1)
                            * Uint512::from(input_token).isqrt(),
                    )
                    .unwrap()
                } else {
                    Uint256::try_from(
                        Uint512::from(input_token) * Uint512::from(pool_info.amount1)
                            / Uint512::from(pool_info.amount0)
                            * Uint512::from(input_token).isqrt(),
                    )
                    .unwrap()
                };
                LIQUIDITY.update(
                    deps.storage,
                    (pool_id_key, receiver.as_bytes()),
                    |liquidity| -> StdResult<_> {
                        Ok(liquidity.unwrap_or_default() + liq / Uint256::from(2u8))
                    },
                )?;
                LIQUIDITY.update(
                    deps.storage,
                    (pool_id_key, liquidity_queue.receiver.as_bytes()),
                    |liquidity| -> StdResult<_> {
                        Ok(liquidity.unwrap_or_default() + liq / Uint256::from(2u8))
                    },
                )?;
                pool_info.total_liquidity += liq;
            }
            if !input_amount.is_zero() {
                if is_chain0 {
                    pool_info.pending_amount0 = input_amount;
                    pool_info.pending_amount1 = Uint256::zero();
                } else {
                    pool_info.pending_amount0 = Uint256::zero();
                    pool_info.pending_amount1 = input_amount;
                }
                LIQUIDITY_QUEUE_IDS.save(
                    deps.storage,
                    pool_id_key,
                    &QueueID {
                        start: 0,
                        length: 1,
                    },
                )?;
                LIQUIDITY_QUEUE.save(
                    deps.storage,
                    (pool_id_key, 0u64.to_be_bytes().as_slice()),
                    &LiquidityQueueElement {
                        chain_id,
                        amount,
                        receiver,
                    },
                )?;
            } else {
                LIQUIDITY_QUEUE_IDS.save(deps.storage, pool_id_key, &liquidity_queue_id)?;
            }
        }
    } else {
        LIQUIDITY_QUEUE.save(
            deps.storage,
            (pool_id_key, 0u64.to_be_bytes().as_slice()),
            &LiquidityQueueElement {
                chain_id,
                amount,
                receiver,
            },
        )?;
        liquidity_queue_id.length = 1;
        LIQUIDITY_QUEUE_IDS.save(deps.storage, pool_id_key, &liquidity_queue_id)?;
        if is_chain0 {
            pool_info.pending_amount0 += amount;
        } else {
            pool_info.pending_amount1 += amount;
        }
    }
    POOLS_INFO.save(deps.storage, pool_id_key, &pool_info)?;
    Ok(Response::new())
}

#[allow(clippy::too_many_arguments)]
fn remove_liquidity(
    deps: DepsMut,
    info: MessageInfo,
    chain0_id: Uint256,
    chain1_id: Uint256,
    token0: String,
    token1: String,
    receiver0: String,
    receiver1: String,
    amount: Uint256,
) -> Result<Response<PalomaMsg>, ContractError> {
    assert!(chain0_id < chain1_id);
    let pool_meta_info = PoolMetaInfo {
        chain0_id,
        chain1_id,
        token0,
        token1,
    };
    let binding = to_binary(&pool_meta_info)?;
    let meta_info_key = binding.as_slice();
    let pool_id = POOL_IDS.load(deps.storage, meta_info_key)?;
    let binding = pool_id.to_be_bytes();
    let pool_id_key = binding.as_slice();
    let mut pool_info = POOLS_INFO.load(deps.storage, pool_id_key)?;
    let amount0 = pool_info
        .amount0
        .multiply_ratio(amount, pool_info.total_liquidity);
    let amount1 = pool_info
        .amount1
        .multiply_ratio(amount, pool_info.total_liquidity);
    pool_info.amount0 = pool_info.amount0.checked_sub(amount0).unwrap();
    pool_info.amount1 = pool_info.amount1.checked_sub(amount1).unwrap();
    pool_info.total_liquidity = pool_info.total_liquidity.checked_sub(amount).unwrap();
    POOLS_INFO.save(deps.storage, pool_id_key, &pool_info)?;
    LIQUIDITY.update(
        deps.storage,
        (pool_id_key, info.sender.as_bytes()),
        |liquidity| -> StdResult<_> {
            Ok(liquidity.unwrap_or_default().checked_sub(amount).unwrap())
        },
    )?;
    #[allow(deprecated)]
    let contract = Contract {
        constructor: None,
        functions: BTreeMap::from_iter(vec![(
            "remove_liquidity".to_string(),
            vec![Function {
                name: "remove_liquidity".to_string(),
                inputs: vec![
                    Param {
                        name: "pool_id".to_string(),
                        kind: ParamType::Uint(256),
                        internal_type: None,
                    },
                    Param {
                        name: "amount".to_string(),
                        kind: ParamType::Uint(256),
                        internal_type: None,
                    },
                    Param {
                        name: "recipient".to_string(),
                        kind: ParamType::Address,
                        internal_type: None,
                    },
                ],
                outputs: Vec::new(),
                constant: None,
                state_mutability: StateMutability::NonPayable,
            }],
        )]),
        events: BTreeMap::new(),
        errors: BTreeMap::new(),
        receive: false,
        fallback: false,
    };

    let mut target_contract_info = TARGET_CONTRACT_INFO.load(deps.storage)?;
    let binding = chain0_id.to_be_bytes();
    let chain_id_key = binding.as_slice();
    let factory = POOL_FACTORIES.load(deps.storage, chain_id_key)?;
    target_contract_info.contract_address = factory;
    target_contract_info.chain_id = chain0_id.to_string();
    let msg0 = CosmosMsg::Custom(PalomaMsg {
        target_contract_info: target_contract_info.clone(),
        payload: Binary(
            contract
                .function("remove_liquidity")
                .unwrap()
                .encode_input(&[
                    Token::Uint(Uint::from_str(chain0_id.to_string().as_str()).unwrap()),
                    Token::Uint(Uint::from_str(amount0.to_string().as_str()).unwrap()),
                    Token::Address(Address::from_str(receiver0.as_str()).unwrap()),
                ])
                .unwrap(),
        ),
    });
    let binding = chain1_id.to_be_bytes();
    let chain_id_key = binding.as_slice();
    let factory = POOL_FACTORIES.load(deps.storage, chain_id_key)?;
    target_contract_info.contract_address = factory;
    target_contract_info.chain_id = chain1_id.to_string();
    let msg1 = CosmosMsg::Custom(PalomaMsg {
        target_contract_info,
        payload: Binary(
            contract
                .function("remove_liquidity")
                .unwrap()
                .encode_input(&[
                    Token::Uint(Uint::from_str(chain1_id.to_string().as_str()).unwrap()),
                    Token::Uint(Uint::from_str(amount1.to_string().as_str()).unwrap()),
                    Token::Address(Address::from_str(receiver1.as_str()).unwrap()),
                ])
                .unwrap(),
        ),
    });
    let response = Response::new().add_message(msg0).add_message(msg1);

    Ok(response)
}

/// Query data from this contract. Currently no query interface is provided.
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, _msg: QueryMsg) -> StdResult<Binary> {
    unimplemented!()
}

#[cfg(test)]
mod tests {}
