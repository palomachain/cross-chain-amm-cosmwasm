//! Execute cross chain transactions.

use crate::ContractError::PoolExists;
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Addr, Binary, CosmosMsg, Deps, DepsMut, Env, Isqrt, MessageInfo, Response,
    StdResult, Uint256, Uint512,
};
use ethabi::{Address, Contract, Function, Param, ParamType, StateMutability, Token, Uint};
use std::cmp::{min, Ordering};
use std::collections::BTreeMap;
use std::ops::{Add, AddAssign, Div, Mul, SubAssign};
use std::str::FromStr;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, PalomaMsg, QueryMsg};
use crate::state::{
    ChainInfo, LiquidityQueueElement, PoolInfo, PoolMetaInfo, QueueID, State, CHAIN_INFO,
    LIQUIDITY, LIQUIDITY_QUEUE, LIQUIDITY_QUEUE_IDS, POOLS_INFO, POOL_IDS, STATE,
};

const MIN_LIQUIDITY: u16 = 1000u16;

/// Instantiate the contract. Initialize the pools.
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    STATE.save(
        deps.storage,
        &State {
            admin: info.sender,
            event_tracker: msg.event_tracker,
            pools_count: Uint256::zero(),
            deadline: msg.deadline,
            fee: 3000,
        },
    )?;
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
            fee,
        } => create_pool(
            deps,
            env,
            chain0_id,
            chain1_id,
            token0,
            token1,
            chain0_init_depositor,
            chain1_init_depositor,
            fee,
        ),
        ExecuteMsg::Swap {
            chain_from_id,
            chain_to_id,
            token_from,
            token_to,
            receiver,
            amount,
        } => swap(
            deps,
            info,
            chain_from_id,
            chain_to_id,
            token_from,
            token_to,
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
        } => add_liquidity(
            deps, info, pool_id, chain_id, token, amount, sender, receiver,
        ),
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
        ExecuteMsg::RegisterChain {
            chain_id,
            chain_name,
            factory,
        } => register_chain(deps, chain_id, chain_name, factory),
        ExecuteMsg::UpdateConfig {
            new_deadline,
            new_fee,
            new_admin,
            new_event_tracker,
        } => {
            STATE.update(deps.storage, |mut state| -> StdResult<_> {
                assert!(state.admin.eq(&info.sender));
                if let Some(new_deadline) = new_deadline {
                    state.deadline = new_deadline;
                }
                if let Some(new_fee) = new_fee {
                    state.fee = new_fee;
                }
                if let Some(new_admin) = new_admin {
                    state.admin = new_admin;
                }
                if let Some(new_event_tracker) = new_event_tracker {
                    state.event_tracker = new_event_tracker;
                }
                Ok(state)
            })?;
            Ok(Response::new())
        }
    }
}

fn register_chain(
    deps: DepsMut,
    chain_id: Uint256,
    chain_name: String,
    factory: String,
) -> Result<Response<PalomaMsg>, ContractError> {
    let binding = chain_id.to_be_bytes();
    let chain_id_key = binding.as_slice();
    assert!(!CHAIN_INFO.has(deps.storage, chain_id_key));
    CHAIN_INFO.save(
        deps.storage,
        chain_id_key,
        &ChainInfo {
            chain_name,
            factory,
        },
    )?;
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
    fee: u16,
) -> Result<Response<PalomaMsg>, ContractError> {
    assert!(chain0_id < chain1_id);
    assert!(fee < 10000);
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
            .plus_seconds(STATE.load(deps.storage)?.deadline)
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
        pool_id = STATE.load(deps.storage)?.pools_count;
    }
    let pool_info = PoolInfo {
        meta: pool_meta_info,
        amount0: Uint256::zero(),
        amount1: Uint256::zero(),
        pending_amount0: Uint256::zero(),
        pending_amount1: Uint256::zero(),
        total_liquidity: Uint256::zero(),
        timestamp: env.block.time,
        chain0_init_depositor,
        chain1_init_depositor,
        fee,
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
    STATE.update(deps.storage, |mut state| -> StdResult<_> {
        state.pools_count.add_assign(Uint256::from(1u8));
        Ok(state)
    })?;
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

    Ok(Response::new()
        .add_message(CosmosMsg::Custom(PalomaMsg {
            job_id: CHAIN_INFO
                .load(deps.storage, chain0_id.to_be_bytes().as_slice())?
                .factory,
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
        }))
        .add_message(CosmosMsg::Custom(PalomaMsg {
            job_id: CHAIN_INFO
                .load(deps.storage, chain1_id.to_be_bytes().as_slice())?
                .factory,
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
        })))
}

fn swap(
    deps: DepsMut,
    info: MessageInfo,
    chain_from_id: Uint256,
    chain_to_id: Uint256,
    token_from: String,
    token_to: String,
    receiver: String,
    amount: Uint256,
) -> Result<Response<PalomaMsg>, ContractError> {
    let msg_sender = STATE.load(deps.storage)?.event_tracker;
    assert!(info.sender.eq(&msg_sender));
    let (chain0_id, chain1_id, token0, token1, is_chain0) = if chain_from_id < chain_to_id {
        (chain_from_id, chain_to_id, token_from, token_to, true)
    } else {
        (chain_to_id, chain_from_id, token_to, token_from, false)
    };
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
    let management_fee = STATE.load(deps.storage)?.fee;
    let management_fee = Uint256::try_from(
        Uint512::from(pool_info.fee)
            .mul(Uint512::from(management_fee))
            .mul(Uint512::from(amount))
            .div(Uint512::from(100_000_000u32)),
    )
        .unwrap();
    let to_amount;
    if is_chain0 {
        to_amount = Uint256::try_from(
            Uint512::from(10_000 - pool_info.fee)
                .mul(Uint512::from(amount))
                .mul(Uint512::from(pool_info.amount1))
                .div(
                    Uint512::from(10_000 - pool_info.fee)
                        .mul(Uint512::from(amount))
                        .add(Uint512::from(pool_info.amount0).mul(Uint512::from(10_000u16))),
                ),
        )
            .unwrap();
        pool_info.amount0.add_assign(amount - management_fee);
        pool_info.amount1.sub_assign(to_amount);
    } else {
        to_amount = Uint256::try_from(
            Uint512::from(10_000 - pool_info.fee)
                .mul(Uint512::from(amount))
                .mul(Uint512::from(pool_info.amount0))
                .div(
                    Uint512::from(10_000 - pool_info.fee)
                        .mul(Uint512::from(amount))
                        .add(Uint512::from(pool_info.amount1).mul(Uint512::from(10_000u16))),
                ),
        )
            .unwrap();
        pool_info.amount1.add_assign(amount - management_fee);
        pool_info.amount0.sub_assign(to_amount);
    }
    POOLS_INFO.save(deps.storage, pool_id_key, &pool_info)?;
    #[allow(deprecated)]
        let contract = Contract {
        constructor: None,
        functions: BTreeMap::from_iter(vec![
            (
                "swap_out".to_string(),
                vec![Function {
                    name: "swap_out".to_string(),
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
            ),
            (
                "withdraw_fee".to_string(),
                vec![Function {
                    name: "withdraw_fee".to_string(),
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
                    ],
                    outputs: Vec::new(),
                    constant: None,
                    state_mutability: StateMutability::NonPayable,
                }],
            ),
        ]),
        events: BTreeMap::new(),
        errors: BTreeMap::new(),
        receive: false,
        fallback: false,
    };

    Ok(Response::new()
        .add_message(CosmosMsg::Custom(PalomaMsg {
            job_id: CHAIN_INFO
                .load(deps.storage, &chain_to_id.to_be_bytes())?
                .factory,
            payload: Binary(
                contract
                    .function("swap_out")
                    .unwrap()
                    .encode_input(&[
                        Token::Uint(Uint::from_str(pool_id.to_string().as_str()).unwrap()),
                        Token::Uint(Uint::from_str(to_amount.to_string().as_str()).unwrap()),
                        Token::Address(Address::from_str(receiver.as_str()).unwrap()),
                    ])
                    .unwrap(),
            ),
        }))
        .add_message(CosmosMsg::Custom(PalomaMsg {
            job_id: CHAIN_INFO
                .load(deps.storage, &chain_from_id.to_be_bytes())?
                .factory,
            payload: Binary(
                contract
                    .function("withdraw_fee")
                    .unwrap()
                    .encode_input(&[
                        Token::Uint(Uint::from_str(pool_id.to_string().as_str()).unwrap()),
                        Token::Uint(Uint::from_str(management_fee.to_string().as_str()).unwrap()),
                    ])
                    .unwrap(),
            ),
        })))
}

fn add_liquidity(
    deps: DepsMut,
    info: MessageInfo,
    pool_id: Uint256,
    chain_id: Uint256,
    token: String,
    amount: Uint256,
    sender: String,
    receiver: Addr,
) -> Result<Response<PalomaMsg>, ContractError> {
    let msg_sender = STATE.load(deps.storage)?.event_tracker;
    assert!(info.sender.eq(&msg_sender));
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
                let (input_token0, input_token1) = match liquidity_queue.amount.cmp(&queue_amount) {
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
                        if is_chain0 {
                            (new_amount, liquidity_queue.amount)
                        } else {
                            (liquidity_queue.amount, new_amount)
                        }
                    }
                    Ordering::Equal => {
                        liquidity_queue_id.length -= id + 1 - liquidity_queue_id.start;
                        liquidity_queue_id.start = id + 1;
                        let new_amount = if is_chain0 {
                            pool_info.pending_amount1 -= queue_amount;
                            Uint256::try_from(
                                Uint512::from(queue_amount) * Uint512::from(pool_info.amount0)
                                    / Uint512::from(pool_info.amount1),
                            )
                                .unwrap()
                        } else {
                            pool_info.pending_amount0 -= queue_amount;
                            Uint256::try_from(
                                Uint512::from(queue_amount) * Uint512::from(pool_info.amount1)
                                    / Uint512::from(pool_info.amount0),
                            )
                                .unwrap()
                        };
                        queue_amount = Uint256::zero();
                        input_amount = Uint256::zero();
                        if is_chain0 {
                            (new_amount, queue_amount)
                        } else {
                            (queue_amount, new_amount)
                        }
                    }
                    Ordering::Greater => {
                        liquidity_queue.amount -= queue_amount;
                        liquidity_queue_id.length -= id - liquidity_queue_id.start;
                        liquidity_queue_id.start = id;

                        let new_amount = if is_chain0 {
                            pool_info.pending_amount1 -= queue_amount;
                            Uint256::try_from(
                                Uint512::from(queue_amount) * Uint512::from(pool_info.amount0)
                                    / Uint512::from(pool_info.amount1),
                            )
                                .unwrap()
                        } else {
                            pool_info.pending_amount0 -= queue_amount;
                            Uint256::try_from(
                                Uint512::from(queue_amount) * Uint512::from(pool_info.amount1)
                                    / Uint512::from(pool_info.amount0),
                            )
                                .unwrap()
                        };

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
                        if is_chain0 {
                            (new_amount, queue_amount)
                        } else {
                            (queue_amount, new_amount)
                        }
                    }
                };

                let liq0 = Uint256::try_from(
                    Uint512::from(input_token0) * Uint512::from(pool_info.total_liquidity)
                        / Uint512::from(pool_info.amount0),
                )
                    .unwrap();
                let liq1 = Uint256::try_from(
                    Uint512::from(input_token1) * Uint512::from(pool_info.total_liquidity)
                        / Uint512::from(pool_info.amount1),
                )
                    .unwrap();
                pool_info.amount0 -= input_token0;
                pool_info.amount1 -= input_token1;
                let liq = min(liq0, liq1);
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

    Ok(Response::new()
        .add_message(CosmosMsg::Custom(PalomaMsg {
            job_id: CHAIN_INFO
                .load(deps.storage, &chain0_id.to_be_bytes())?
                .factory,
            payload: Binary(
                contract
                    .function("remove_liquidity")
                    .unwrap()
                    .encode_input(&[
                        Token::Uint(Uint::from_str(pool_id.to_string().as_str()).unwrap()),
                        Token::Uint(Uint::from_str(amount0.to_string().as_str()).unwrap()),
                        Token::Address(Address::from_str(receiver0.as_str()).unwrap()),
                    ])
                    .unwrap(),
            ),
        }))
        .add_message(CosmosMsg::Custom(PalomaMsg {
            job_id: CHAIN_INFO
                .load(deps.storage, &chain1_id.to_be_bytes())?
                .factory,
            payload: Binary(
                contract
                    .function("remove_liquidity")
                    .unwrap()
                    .encode_input(&[
                        Token::Uint(Uint::from_str(pool_id.to_string().as_str()).unwrap()),
                        Token::Uint(Uint::from_str(amount1.to_string().as_str()).unwrap()),
                        Token::Address(Address::from_str(receiver1.as_str()).unwrap()),
                    ])
                    .unwrap(),
            ),
        })))
}

/// Query data from this contract. Currently no query interface is provided.
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::ChainInfo { chain_id } => {
            to_binary(&CHAIN_INFO.load(deps.storage, chain_id.to_be_bytes().as_slice())?)
        }
        QueryMsg::PoolId {
            chain0_id,
            chain1_id,
            token0,
            token1,
        } => {
            assert!(chain0_id < chain1_id);
            let pool_meta_info = PoolMetaInfo {
                chain0_id,
                chain1_id,
                token0,
                token1,
            };
            let binding = to_binary(&pool_meta_info)?;
            let meta_info_key = binding.as_slice();
            to_binary(&POOL_IDS.load(deps.storage, meta_info_key)?)
        }
        QueryMsg::PoolInfo { pool_id } => {
            to_binary(&POOLS_INFO.load(deps.storage, pool_id.to_be_bytes().as_slice())?)
        }
        QueryMsg::State {} => to_binary(&STATE.load(deps.storage)?),
        QueryMsg::LiquidityQueue { pool_id } => {
            to_binary(&LIQUIDITY_QUEUE_IDS.load(deps.storage, pool_id.to_be_bytes().as_slice())?)
        }
        QueryMsg::LiquidityQueueElement { pool_id, queue_id } => to_binary(&LIQUIDITY_QUEUE.load(
            deps.storage,
            (
                pool_id.to_be_bytes().as_slice(),
                queue_id.to_be_bytes().as_slice(),
            ),
        )?),
    }
}

#[cfg(test)]
mod tests {}
