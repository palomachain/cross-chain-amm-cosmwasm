//! Smoke tests.

use crate::contract::{execute, instantiate, query};
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{LiquidityQueueElement, PoolInfo, QueueID, State};
use crate::ContractError;
use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{from_binary, to_binary, Addr, CosmosMsg, Isqrt, Uint256, Uint512};

const MIN_LIQUIDITY: u128 = 1000u128;

/// Test instantiating the contract, creating a pool, adding liquidity and making a trade.
#[test]
fn happy_path() -> Result<(), ContractError> {
    let mut deps = mock_dependencies();

    let info = mock_info("admin0000", &[]);
    let _ = instantiate(
        deps.as_mut(),
        mock_env(),
        info,
        InstantiateMsg {
            event_tracker: Addr::unchecked("admin0000".to_string()),
            deadline: 1000,
        },
    )?;
    let info = mock_info("admin0000", &[]);
    let (chain0_id, chain1_id) = (42u32.into(), 52u32.into());
    let (token0, token1) = (
        "0123456789012345678901234567890123456789".to_string(),
        "abcdefabcdefabcdefabcdefabcdefabcdefabcd".to_string(),
    );
    let (sender0, sender1) = ("addr01234".to_string(), "addr98765".to_string());

    for chain_id in [chain0_id, chain1_id] {
        let r = execute(
            deps.as_mut(),
            mock_env(),
            info.clone(),
            ExecuteMsg::RegisterJobId {
                chain_id,
                job: "create_pool".to_string(),
                job_id: "00000".to_string(),
            },
        )?;
        assert_eq!(r.messages.len(), 0);
    }

    let r = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        ExecuteMsg::CreatePool {
            chain0_id,
            chain1_id,
            token0: token0.clone(),
            token1: token1.clone(),
            chain0_init_depositor: sender0.clone(),
            chain1_init_depositor: sender1.clone(),
            fee: 30,
        },
    )?;
    assert_eq!(r.messages.len(), 2);

    for (chain_id, token, sender) in [
        (chain0_id.clone(), token0.clone(), sender0.clone()),
        (chain1_id.clone(), token1.clone(), sender1.clone()),
    ] {
        let r = execute(
            deps.as_mut(),
            mock_env(),
            info.clone(),
            ExecuteMsg::AddLiquidity {
                pool_id: 0u32.into(),
                chain_id,
                token,
                amount: 10000u32.into(),
                sender,
                receiver: Addr::unchecked("addr01234"),
            },
        )?;
        assert_eq!(r.messages.len(), 0);
    }
    Ok(())
}

#[test]
fn initialize_test() -> Result<(), ContractError> {
    let mut deps = mock_dependencies();
    let info = mock_info("admin0", &[]);
    let _ = instantiate(
        deps.as_mut(),
        mock_env(),
        info,
        InstantiateMsg {
            event_tracker: Addr::unchecked("tracker0".to_string()),
            deadline: 1000,
        },
    )?;
    let res = query(deps.as_ref(), mock_env(), QueryMsg::State {})?;
    let expected = to_binary(&State {
        admin: Addr::unchecked("admin0"),
        event_tracker: Addr::unchecked("tracker0"),
        pools_count: Uint256::zero(),
        deadline: 1000,
        fee: 3000,
    })
    .unwrap();
    assert_eq!(res, expected);
    Ok(())
}

#[test]
fn register_job_test() -> Result<(), ContractError> {
    let mut deps = mock_dependencies();
    let info = mock_info("admin0", &[]);
    let _ = instantiate(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        InstantiateMsg {
            event_tracker: Addr::unchecked("tracker0".to_string()),
            deadline: 1000,
        },
    )?;
    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        ExecuteMsg::RegisterJobId {
            chain_id: Uint256::from(0u8),
            job: "create_pool".to_string(),
            job_id: "create_pool_eth".to_string(),
        },
    )?;
    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::JobInfo {
            chain_id: Uint256::from(0u8),
            job: "create_pool".to_string(),
        },
    )?;
    let expected = to_binary(&"create_pool_eth".to_string()).unwrap();
    assert_eq!(res, expected);
    Ok(())
}

#[test]
fn create_pool_test_0() -> Result<(), ContractError> {
    let mut deps = mock_dependencies();
    let info = mock_info("admin0", &[]);
    let _ = instantiate(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        InstantiateMsg {
            event_tracker: Addr::unchecked("tracker0".to_string()),
            deadline: 1000,
        },
    )?;
    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        ExecuteMsg::RegisterJobId {
            chain_id: Uint256::from(0u8),
            job: "create_pool".to_string(),
            job_id: "create_pool_eth".to_string(),
        },
    )?;
    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        ExecuteMsg::RegisterJobId {
            chain_id: Uint256::from(1u8),
            job: "create_pool".to_string(),
            job_id: "create_pool_bnb".to_string(),
        },
    )?;
    let res = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        ExecuteMsg::CreatePool {
            chain0_id: Uint256::from(0u8),
            chain1_id: Uint256::from(0u8),
            token0: "0x0000000000000000000000000000000000000000".to_string(),
            token1: "0x0000000000000000000000000000000000000000".to_string(),
            chain0_init_depositor: "0x0000000000000000000000000000000000000001".to_string(),
            chain1_init_depositor: "0x0000000000000000000000000000000000000001".to_string(),
            fee: 3000,
        },
    )?;
    assert_eq!(res.messages.len(), 2);
    let mut iter = res.messages.iter();
    while let Some(msg) = iter.next() {
        if let CosmosMsg::Custom(data) = msg.msg.clone() {
            assert_eq!("create_pool_eth".to_string(), data.job_id);
        }
    }
    Ok(())
}

#[test]
fn create_pool_test_1() -> Result<(), ContractError> {
    let mut deps = mock_dependencies();
    let info = mock_info("admin0", &[]);
    let _ = instantiate(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        InstantiateMsg {
            event_tracker: Addr::unchecked("tracker0".to_string()),
            deadline: 1000,
        },
    )?;
    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        ExecuteMsg::RegisterJobId {
            chain_id: Uint256::from(0u8),
            job: "create_pool".to_string(),
            job_id: "create_pool_eth".to_string(),
        },
    )?;
    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        ExecuteMsg::RegisterJobId {
            chain_id: Uint256::from(1u8),
            job: "create_pool".to_string(),
            job_id: "create_pool_bnb".to_string(),
        },
    )?;
    let res = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        ExecuteMsg::CreatePool {
            chain0_id: Uint256::from(0u8),
            chain1_id: Uint256::from(0u8),
            token0: "0x0000000000000000000000000000000000000001".to_string(),
            token1: "0x0000000000000000000000000000000000000000".to_string(),
            chain0_init_depositor: "0x0000000000000000000000000000000000000001".to_string(),
            chain1_init_depositor: "0x0000000000000000000000000000000000000001".to_string(),
            fee: 3000,
        },
    )?;
    assert_eq!(res.messages.len(), 2);
    let mut iter = res.messages.iter();
    while let Some(msg) = iter.next() {
        if let CosmosMsg::Custom(data) = msg.msg.clone() {
            assert_eq!("create_pool_eth".to_string(), data.job_id);
        }
    }
    let mut env = mock_env();
    env.block.time = env.block.time.plus_seconds(2000u64);
    let _ = execute(
        deps.as_mut(),
        env,
        info.clone(),
        ExecuteMsg::CreatePool {
            chain0_id: Uint256::from(0u8),
            chain1_id: Uint256::from(0u8),
            token0: "0x0000000000000000000000000000000000000001".to_string(),
            token1: "0x0000000000000000000000000000000000000000".to_string(),
            chain0_init_depositor: "0x0000000000000000000000000000000000000001".to_string(),
            chain1_init_depositor: "0x0000000000000000000000000000000000000001".to_string(),
            fee: 3000,
        },
    )?;
    Ok(())
}

#[test]
fn create_pool_test_2() -> Result<(), ContractError> {
    let mut deps = mock_dependencies();
    let info = mock_info("admin0", &[]);
    let _ = instantiate(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        InstantiateMsg {
            event_tracker: Addr::unchecked("tracker0".to_string()),
            deadline: 1000,
        },
    )?;
    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        ExecuteMsg::RegisterJobId {
            chain_id: Uint256::from(0u8),
            job: "create_pool".to_string(),
            job_id: "create_pool_eth".to_string(),
        },
    )?;
    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        ExecuteMsg::RegisterJobId {
            chain_id: Uint256::from(1u8),
            job: "create_pool".to_string(),
            job_id: "create_pool_bnb".to_string(),
        },
    )?;
    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        ExecuteMsg::CreatePool {
            chain0_id: Uint256::from(0u8),
            chain1_id: Uint256::from(0u8),
            token0: "0x0000000000000000000000000000000000000000".to_string(),
            token1: "0x0000000000000000000000000000000000000000".to_string(),
            chain0_init_depositor: "0x0000000000000000000000000000000000000001".to_string(),
            chain1_init_depositor: "0x0000000000000000000000000000000000000001".to_string(),
            fee: 3000,
        },
    )?;
    let res = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        ExecuteMsg::CreatePool {
            chain0_id: Uint256::from(0u8),
            chain1_id: Uint256::from(0u8),
            token0: "0x0000000000000000000000000000000000000000".to_string(),
            token1: "0x0000000000000000000000000000000000000000".to_string(),
            chain0_init_depositor: "0x0000000000000000000000000000000000000001".to_string(),
            chain1_init_depositor: "0x0000000000000000000000000000000000000001".to_string(),
            fee: 3000,
        },
    );
    assert!(res.err().is_some());
    Ok(())
}

/// Create pool chain(0)_token(base coin) - chain(1)_token(base coin)
/// and add liquidity same amount(1e18).
#[test]
fn add_liquidity_test_0() -> Result<(), ContractError> {
    let mut deps = mock_dependencies();
    let info = mock_info("admin0", &[]);
    let _ = instantiate(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        InstantiateMsg {
            event_tracker: Addr::unchecked("tracker0".to_string()),
            deadline: 1000,
        },
    )?;
    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        ExecuteMsg::RegisterJobId {
            chain_id: Uint256::from(0u8),
            job: "create_pool".to_string(),
            job_id: "create_pool_eth".to_string(),
        },
    )?;
    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        ExecuteMsg::RegisterJobId {
            chain_id: Uint256::from(1u8),
            job: "create_pool".to_string(),
            job_id: "create_pool_bnb".to_string(),
        },
    )?;
    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        ExecuteMsg::CreatePool {
            chain0_id: Uint256::from(0u8),
            chain1_id: Uint256::from(1u8),
            token0: "0x0000000000000000000000000000000000000000".to_string(),
            token1: "0x0000000000000000000000000000000000000000".to_string(),
            chain0_init_depositor: "0x0000000000000000000000000000000000000001".to_string(),
            chain1_init_depositor: "0x0000000000000000000000000000000000000001".to_string(),
            fee: 3000,
        },
    )?;
    let pool_id: Uint256 = from_binary(&query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::PoolId {
            chain0_id: Uint256::from(0u8),
            chain1_id: Uint256::from(1u8),
            token0: "0x0000000000000000000000000000000000000000".to_string(),
            token1: "0x0000000000000000000000000000000000000000".to_string(),
        },
    )?)
    .unwrap();
    let info = mock_info("tracker0", &[]);
    let token_amount = 1_000_000_000_000_000_000u128;
    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        ExecuteMsg::AddLiquidity {
            pool_id,
            chain_id: Uint256::from(0u8),
            token: "0x0000000000000000000000000000000000000000".to_string(),
            amount: Uint256::from(token_amount),
            sender: "0x0000000000000000000000000000000000000001".to_string(),
            receiver: Addr::unchecked("liquidity_adder"),
        },
    )?;
    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        ExecuteMsg::AddLiquidity {
            pool_id,
            chain_id: Uint256::from(1u8),
            token: "0x0000000000000000000000000000000000000000".to_string(),
            amount: Uint256::from(token_amount),
            sender: "0x0000000000000000000000000000000000000001".to_string(),
            receiver: Addr::unchecked("liquidity_adder"),
        },
    )?;
    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::Liquidity {
            pool_id: Default::default(),
            owner: Addr::unchecked("liquidity_adder"),
        },
    )?;
    let expected = to_binary(&Uint256::from(token_amount - MIN_LIQUIDITY)).unwrap();
    assert_eq!(res, expected);
    Ok(())
}

/// Create pool chain(0)_token(base coin) - chain(1)_token(0x1234567890123456789012345678901234567890)
/// and add liquidity same amount(1e18).
#[test]
fn add_liquidity_test_1() -> Result<(), ContractError> {
    let mut deps = mock_dependencies();
    let info = mock_info("admin0", &[]);
    let _ = instantiate(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        InstantiateMsg {
            event_tracker: Addr::unchecked("tracker0".to_string()),
            deadline: 1000,
        },
    )?;
    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        ExecuteMsg::RegisterJobId {
            chain_id: Uint256::from(0u8),
            job: "create_pool".to_string(),
            job_id: "create_pool_eth".to_string(),
        },
    )?;
    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        ExecuteMsg::RegisterJobId {
            chain_id: Uint256::from(1u8),
            job: "create_pool".to_string(),
            job_id: "create_pool_bnb".to_string(),
        },
    )?;
    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        ExecuteMsg::CreatePool {
            chain0_id: Uint256::from(0u8),
            chain1_id: Uint256::from(1u8),
            token0: "0x0000000000000000000000000000000000000000".to_string(),
            token1: "0x1234567890123456789012345678901234567890".to_string(),
            chain0_init_depositor: "0x0000000000000000000000000000000000000001".to_string(),
            chain1_init_depositor: "0x0000000000000000000000000000000000000001".to_string(),
            fee: 3000,
        },
    )?;
    let info = mock_info("tracker0", &[]);
    let token_amount = 1_000_000_000_000_000_000u128;
    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        ExecuteMsg::AddLiquidity {
            pool_id: Uint256::from(0u8),
            chain_id: Uint256::from(0u8),
            token: "0x0000000000000000000000000000000000000000".to_string(),
            amount: Uint256::from(token_amount),
            sender: "0x0000000000000000000000000000000000000001".to_string(),
            receiver: Addr::unchecked("liquidity_adder"),
        },
    )?;
    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        ExecuteMsg::AddLiquidity {
            pool_id: Uint256::from(0u8),
            chain_id: Uint256::from(1u8),
            token: "0x1234567890123456789012345678901234567890".to_string(),
            amount: Uint256::from(token_amount),
            sender: "0x0000000000000000000000000000000000000001".to_string(),
            receiver: Addr::unchecked("liquidity_adder"),
        },
    )?;
    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::Liquidity {
            pool_id: Default::default(),
            owner: Addr::unchecked("liquidity_adder"),
        },
    )?;
    let expected = to_binary(&Uint256::from(token_amount - MIN_LIQUIDITY)).unwrap();
    assert_eq!(res, expected);
    Ok(())
}

/// Create pool chain(0)_token(base coin) - chain(1)_token(base coin)
/// and add liquidity different amount.
#[test]
fn add_liquidity_test_2() -> Result<(), ContractError> {
    let mut deps = mock_dependencies();
    let info = mock_info("admin0", &[]);
    let _ = instantiate(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        InstantiateMsg {
            event_tracker: Addr::unchecked("tracker0".to_string()),
            deadline: 1000,
        },
    )?;
    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        ExecuteMsg::RegisterJobId {
            chain_id: Uint256::from(0u8),
            job: "create_pool".to_string(),
            job_id: "create_pool_eth".to_string(),
        },
    )?;
    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        ExecuteMsg::RegisterJobId {
            chain_id: Uint256::from(1u8),
            job: "create_pool".to_string(),
            job_id: "create_pool_bnb".to_string(),
        },
    )?;
    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        ExecuteMsg::CreatePool {
            chain0_id: Uint256::from(0u8),
            chain1_id: Uint256::from(1u8),
            token0: "0x0000000000000000000000000000000000000000".to_string(),
            token1: "0x0000000000000000000000000000000000000000".to_string(),
            chain0_init_depositor: "0x0000000000000000000000000000000000000001".to_string(),
            chain1_init_depositor: "0x0000000000000000000000000000000000000001".to_string(),
            fee: 3000,
        },
    )?;
    let info = mock_info("tracker0", &[]);
    let token0_amount = 1_000_000_000_000_000_000u128;
    let token1_amount = 2_000_000_000_000_000_000u128;
    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        ExecuteMsg::AddLiquidity {
            pool_id: Uint256::from(0u8),
            chain_id: Uint256::from(0u8),
            token: "0x0000000000000000000000000000000000000000".to_string(),
            amount: Uint256::from(token0_amount),
            sender: "0x0000000000000000000000000000000000000001".to_string(),
            receiver: Addr::unchecked("liquidity_adder"),
        },
    )?;
    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        ExecuteMsg::AddLiquidity {
            pool_id: Uint256::from(0u8),
            chain_id: Uint256::from(1u8),
            token: "0x0000000000000000000000000000000000000000".to_string(),
            amount: Uint256::from(token1_amount),
            sender: "0x0000000000000000000000000000000000000001".to_string(),
            receiver: Addr::unchecked("liquidity_adder"),
        },
    )?;
    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::Liquidity {
            pool_id: Default::default(),
            owner: Addr::unchecked("liquidity_adder"),
        },
    )?;
    let expected = to_binary(&Uint256::from(
        (token0_amount * token1_amount).isqrt() - MIN_LIQUIDITY,
    ))
    .unwrap();
    assert_eq!(res, expected);
    Ok(())
}

/// Create pool chain(0)_token(base coin) - chain(1)_token(base coin)
/// add liquidity into queue
#[test]
fn add_liquidity_test_3() -> Result<(), ContractError> {
    let mut deps = mock_dependencies();
    let info = mock_info("admin0", &[]);
    let _ = instantiate(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        InstantiateMsg {
            event_tracker: Addr::unchecked("tracker0".to_string()),
            deadline: 1000,
        },
    )?;
    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        ExecuteMsg::RegisterJobId {
            chain_id: Uint256::from(0u8),
            job: "create_pool".to_string(),
            job_id: "create_pool_eth".to_string(),
        },
    )?;
    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        ExecuteMsg::RegisterJobId {
            chain_id: Uint256::from(1u8),
            job: "create_pool".to_string(),
            job_id: "create_pool_bnb".to_string(),
        },
    )?;
    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        ExecuteMsg::CreatePool {
            chain0_id: Uint256::from(0u8),
            chain1_id: Uint256::from(1u8),
            token0: "0x0000000000000000000000000000000000000000".to_string(),
            token1: "0x0000000000000000000000000000000000000000".to_string(),
            chain0_init_depositor: "0x0000000000000000000000000000000000000001".to_string(),
            chain1_init_depositor: "0x0000000000000000000000000000000000000001".to_string(),
            fee: 3000,
        },
    )?;
    let info = mock_info("tracker0", &[]);
    let token0_amount = 1_000_000_000_000_000_000u128;
    let token1_amount = 2_000_000_000_000_000_000u128;
    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        ExecuteMsg::AddLiquidity {
            pool_id: Uint256::from(0u8),
            chain_id: Uint256::from(0u8),
            token: "0x0000000000000000000000000000000000000000".to_string(),
            amount: Uint256::from(token0_amount),
            sender: "0x0000000000000000000000000000000000000001".to_string(),
            receiver: Addr::unchecked("liquidity_adder"),
        },
    )?;
    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        ExecuteMsg::AddLiquidity {
            pool_id: Uint256::from(0u8),
            chain_id: Uint256::from(1u8),
            token: "0x0000000000000000000000000000000000000000".to_string(),
            amount: Uint256::from(token1_amount),
            sender: "0x0000000000000000000000000000000000000001".to_string(),
            receiver: Addr::unchecked("liquidity_adder"),
        },
    )?;

    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        ExecuteMsg::AddLiquidity {
            pool_id: Uint256::from(0u8),
            chain_id: Uint256::from(0u8),
            token: "0x0000000000000000000000000000000000000000".to_string(),
            amount: Uint256::from(token0_amount),
            sender: "0x0000000000000000000000000000000000000001".to_string(),
            receiver: Addr::unchecked("liquidity_adder"),
        },
    )?;

    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::LiquidityQueueElement {
            pool_id: Default::default(),
            queue_id: 0,
        },
    )?;
    let queue_element: LiquidityQueueElement = from_binary(&res).unwrap();
    assert_eq!(queue_element.amount, Uint256::from(token0_amount));
    assert_eq!(queue_element.receiver, Addr::unchecked("liquidity_adder"));

    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        ExecuteMsg::AddLiquidity {
            pool_id: Uint256::from(0u8),
            chain_id: Uint256::from(0u8),
            token: "0x0000000000000000000000000000000000000000".to_string(),
            amount: Uint256::from(token0_amount),
            sender: "0x0000000000000000000000000000000000000001".to_string(),
            receiver: Addr::unchecked("liquidity_adder"),
        },
    )?;

    Ok(())
}

/// Create pool chain(0)_token(base coin) - chain(1)_token(base coin)
/// add liquidity into queue
#[test]
fn add_liquidity_test_3_1() -> Result<(), ContractError> {
    let mut deps = mock_dependencies();
    let info = mock_info("admin0", &[]);
    let _ = instantiate(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        InstantiateMsg {
            event_tracker: Addr::unchecked("tracker0".to_string()),
            deadline: 1000,
        },
    )?;
    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        ExecuteMsg::RegisterJobId {
            chain_id: Uint256::from(0u8),
            job: "create_pool".to_string(),
            job_id: "create_pool_eth".to_string(),
        },
    )?;
    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        ExecuteMsg::RegisterJobId {
            chain_id: Uint256::from(1u8),
            job: "create_pool".to_string(),
            job_id: "create_pool_bnb".to_string(),
        },
    )?;
    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        ExecuteMsg::CreatePool {
            chain0_id: Uint256::from(0u8),
            chain1_id: Uint256::from(1u8),
            token0: "0x0000000000000000000000000000000000000000".to_string(),
            token1: "0x0000000000000000000000000000000000000000".to_string(),
            chain0_init_depositor: "0x0000000000000000000000000000000000000001".to_string(),
            chain1_init_depositor: "0x0000000000000000000000000000000000000001".to_string(),
            fee: 3000,
        },
    )?;
    let info = mock_info("tracker0", &[]);
    let token0_amount = 1_000_000_000_000_000_000u128;
    let token1_amount = 2_000_000_000_000_000_000u128;
    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        ExecuteMsg::AddLiquidity {
            pool_id: Uint256::from(0u8),
            chain_id: Uint256::from(1u8),
            token: "0x0000000000000000000000000000000000000000".to_string(),
            amount: Uint256::from(token1_amount),
            sender: "0x0000000000000000000000000000000000000001".to_string(),
            receiver: Addr::unchecked("liquidity_adder"),
        },
    )?;

    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        ExecuteMsg::AddLiquidity {
            pool_id: Uint256::from(0u8),
            chain_id: Uint256::from(0u8),
            token: "0x0000000000000000000000000000000000000000".to_string(),
            amount: Uint256::from(token0_amount),
            sender: "0x0000000000000000000000000000000000000001".to_string(),
            receiver: Addr::unchecked("liquidity_adder0"),
        },
    )?;

    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        ExecuteMsg::AddLiquidity {
            pool_id: Uint256::from(0u8),
            chain_id: Uint256::from(1u8),
            token: "0x0000000000000000000000000000000000000000".to_string(),
            amount: Uint256::from(token1_amount),
            sender: "0x0000000000000000000000000000000000000001".to_string(),
            receiver: Addr::unchecked("liquidity_adder"),
        },
    )?;

    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::LiquidityQueueElement {
            pool_id: Default::default(),
            queue_id: 0,
        },
    )?;
    let queue_element: LiquidityQueueElement = from_binary(&res).unwrap();
    assert_eq!(queue_element.amount, Uint256::from(token1_amount));
    assert_eq!(queue_element.receiver, Addr::unchecked("liquidity_adder"));

    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        ExecuteMsg::AddLiquidity {
            pool_id: Uint256::from(0u8),
            chain_id: Uint256::from(1u8),
            token: "0x0000000000000000000000000000000000000000".to_string(),
            amount: Uint256::from(token1_amount),
            sender: "0x0000000000000000000000000000000000000001".to_string(),
            receiver: Addr::unchecked("liquidity_adder"),
        },
    )?;

    Ok(())
}

/// Create pool chain(0)_token(base coin) - chain(1)_token(base coin)
/// add liquidity into queue twice
#[test]
fn add_liquidity_test_4() -> Result<(), ContractError> {
    let mut deps = mock_dependencies();
    let info = mock_info("admin0", &[]);
    let _ = instantiate(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        InstantiateMsg {
            event_tracker: Addr::unchecked("tracker0".to_string()),
            deadline: 1000,
        },
    )?;
    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        ExecuteMsg::RegisterJobId {
            chain_id: Uint256::from(0u8),
            job: "create_pool".to_string(),
            job_id: "create_pool_eth".to_string(),
        },
    )?;
    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        ExecuteMsg::RegisterJobId {
            chain_id: Uint256::from(1u8),
            job: "create_pool".to_string(),
            job_id: "create_pool_bnb".to_string(),
        },
    )?;
    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        ExecuteMsg::CreatePool {
            chain0_id: Uint256::from(0u8),
            chain1_id: Uint256::from(1u8),
            token0: "0x0000000000000000000000000000000000000000".to_string(),
            token1: "0x0000000000000000000000000000000000000000".to_string(),
            chain0_init_depositor: "0x0000000000000000000000000000000000000001".to_string(),
            chain1_init_depositor: "0x0000000000000000000000000000000000000001".to_string(),
            fee: 3000,
        },
    )?;
    let info = mock_info("tracker0", &[]);
    let token0_amount = 1_000_000_000_000_000_000u128;
    let token1_amount = 2_000_000_000_000_000_000u128;
    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        ExecuteMsg::AddLiquidity {
            pool_id: Uint256::from(0u8),
            chain_id: Uint256::from(0u8),
            token: "0x0000000000000000000000000000000000000000".to_string(),
            amount: Uint256::from(token0_amount),
            sender: "0x0000000000000000000000000000000000000001".to_string(),
            receiver: Addr::unchecked("liquidity_adder"),
        },
    )?;
    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        ExecuteMsg::AddLiquidity {
            pool_id: Uint256::from(0u8),
            chain_id: Uint256::from(1u8),
            token: "0x0000000000000000000000000000000000000000".to_string(),
            amount: Uint256::from(token1_amount),
            sender: "0x0000000000000000000000000000000000000001".to_string(),
            receiver: Addr::unchecked("liquidity_adder"),
        },
    )?;

    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        ExecuteMsg::AddLiquidity {
            pool_id: Uint256::from(0u8),
            chain_id: Uint256::from(0u8),
            token: "0x0000000000000000000000000000000000000000".to_string(),
            amount: Uint256::from(token0_amount),
            sender: "0x0000000000000000000000000000000000000001".to_string(),
            receiver: Addr::unchecked("liquidity_adder"),
        },
    )?;
    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        ExecuteMsg::AddLiquidity {
            pool_id: Uint256::from(0u8),
            chain_id: Uint256::from(1u8),
            token: "0x0000000000000000000000000000000000000000".to_string(),
            amount: Uint256::from(token1_amount),
            sender: "0x0000000000000000000000000000000000000001".to_string(),
            receiver: Addr::unchecked("liquidity_adder"),
        },
    )?;
    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::LiquidityQueue {
            pool_id: Default::default(),
        },
    )?;
    let queue_id: QueueID = from_binary(&res).unwrap();
    assert_eq!(queue_id.start, 1);
    assert_eq!(queue_id.length, 0);
    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::PoolInfo {
            pool_id: Default::default(),
        },
    )?;
    let pool_info: PoolInfo = from_binary(&res).unwrap();
    assert_eq!(
        Uint256::from((token0_amount * token1_amount).isqrt() * 2),
        pool_info.total_liquidity
    );
    assert_eq!(pool_info.fee, 3000);
    assert_eq!(pool_info.amount0, Uint256::from(token0_amount * 2));
    assert_eq!(pool_info.amount1, Uint256::from(token1_amount * 2));
    assert_eq!(pool_info.pending_amount0, Uint256::zero());
    assert_eq!(pool_info.pending_amount1, Uint256::zero());
    Ok(())
}

/// Create pool chain(0)_token(base coin) - chain(1)_token(base coin)
/// add liquidity into queue twice
#[test]
fn add_liquidity_test_4_1() -> Result<(), ContractError> {
    let mut deps = mock_dependencies();
    let info = mock_info("admin0", &[]);
    let _ = instantiate(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        InstantiateMsg {
            event_tracker: Addr::unchecked("tracker0".to_string()),
            deadline: 1000,
        },
    )?;
    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        ExecuteMsg::RegisterJobId {
            chain_id: Uint256::from(0u8),
            job: "create_pool".to_string(),
            job_id: "create_pool_eth".to_string(),
        },
    )?;
    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        ExecuteMsg::RegisterJobId {
            chain_id: Uint256::from(1u8),
            job: "create_pool".to_string(),
            job_id: "create_pool_bnb".to_string(),
        },
    )?;
    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        ExecuteMsg::CreatePool {
            chain0_id: Uint256::from(0u8),
            chain1_id: Uint256::from(1u8),
            token0: "0x0000000000000000000000000000000000000000".to_string(),
            token1: "0x0000000000000000000000000000000000000000".to_string(),
            chain0_init_depositor: "0x0000000000000000000000000000000000000001".to_string(),
            chain1_init_depositor: "0x0000000000000000000000000000000000000001".to_string(),
            fee: 3000,
        },
    )?;
    let info = mock_info("tracker0", &[]);
    let token0_amount = 1_000_000_000_000_000_000u128;
    let token1_amount = 2_000_000_000_000_000_000u128;
    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        ExecuteMsg::AddLiquidity {
            pool_id: Uint256::from(0u8),
            chain_id: Uint256::from(0u8),
            token: "0x0000000000000000000000000000000000000000".to_string(),
            amount: Uint256::from(token0_amount),
            sender: "0x0000000000000000000000000000000000000001".to_string(),
            receiver: Addr::unchecked("liquidity_adder"),
        },
    )?;
    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        ExecuteMsg::AddLiquidity {
            pool_id: Uint256::from(0u8),
            chain_id: Uint256::from(1u8),
            token: "0x0000000000000000000000000000000000000000".to_string(),
            amount: Uint256::from(token1_amount),
            sender: "0x0000000000000000000000000000000000000001".to_string(),
            receiver: Addr::unchecked("liquidity_adder"),
        },
    )?;

    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        ExecuteMsg::AddLiquidity {
            pool_id: Uint256::from(0u8),
            chain_id: Uint256::from(1u8),
            token: "0x0000000000000000000000000000000000000000".to_string(),
            amount: Uint256::from(token1_amount),
            sender: "0x0000000000000000000000000000000000000001".to_string(),
            receiver: Addr::unchecked("liquidity_adder"),
        },
    )?;
    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        ExecuteMsg::AddLiquidity {
            pool_id: Uint256::from(0u8),
            chain_id: Uint256::from(0u8),
            token: "0x0000000000000000000000000000000000000000".to_string(),
            amount: Uint256::from(token0_amount),
            sender: "0x0000000000000000000000000000000000000001".to_string(),
            receiver: Addr::unchecked("liquidity_adder"),
        },
    )?;
    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::LiquidityQueue {
            pool_id: Default::default(),
        },
    )?;
    let queue_id: QueueID = from_binary(&res).unwrap();
    assert_eq!(queue_id.start, 1);
    assert_eq!(queue_id.length, 0);
    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::PoolInfo {
            pool_id: Default::default(),
        },
    )?;
    let pool_info: PoolInfo = from_binary(&res).unwrap();
    assert_eq!(
        Uint256::from((token0_amount * token1_amount).isqrt() * 2),
        pool_info.total_liquidity
    );
    assert_eq!(pool_info.fee, 3000);
    assert_eq!(pool_info.amount0, Uint256::from(token0_amount * 2));
    assert_eq!(pool_info.amount1, Uint256::from(token1_amount * 2));
    assert_eq!(pool_info.pending_amount0, Uint256::zero());
    assert_eq!(pool_info.pending_amount1, Uint256::zero());
    Ok(())
}

/// Create pool chain(0)_token(base coin) - chain(1)_token(base coin)
/// add liquidity into queue different amounts
#[test]
fn add_liquidity_test_5() -> Result<(), ContractError> {
    let mut deps = mock_dependencies();
    let info = mock_info("admin0", &[]);
    let _ = instantiate(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        InstantiateMsg {
            event_tracker: Addr::unchecked("tracker0".to_string()),
            deadline: 1000,
        },
    )?;
    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        ExecuteMsg::RegisterJobId {
            chain_id: Uint256::from(0u8),
            job: "create_pool".to_string(),
            job_id: "create_pool_eth".to_string(),
        },
    )?;
    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        ExecuteMsg::RegisterJobId {
            chain_id: Uint256::from(1u8),
            job: "create_pool".to_string(),
            job_id: "create_pool_bnb".to_string(),
        },
    )?;
    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        ExecuteMsg::CreatePool {
            chain0_id: Uint256::from(0u8),
            chain1_id: Uint256::from(1u8),
            token0: "0x0000000000000000000000000000000000000000".to_string(),
            token1: "0x0000000000000000000000000000000000000000".to_string(),
            chain0_init_depositor: "0x0000000000000000000000000000000000000001".to_string(),
            chain1_init_depositor: "0x0000000000000000000000000000000000000001".to_string(),
            fee: 3000,
        },
    )?;
    let info = mock_info("tracker0", &[]);
    let token0_amount = 1_000_000_000_000_000_000u128;
    let token1_amount = 2_000_000_000_000_000_000u128;
    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        ExecuteMsg::AddLiquidity {
            pool_id: Uint256::from(0u8),
            chain_id: Uint256::from(0u8),
            token: "0x0000000000000000000000000000000000000000".to_string(),
            amount: Uint256::from(token0_amount),
            sender: "0x0000000000000000000000000000000000000001".to_string(),
            receiver: Addr::unchecked("liquidity_adder"),
        },
    )?;
    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        ExecuteMsg::AddLiquidity {
            pool_id: Uint256::from(0u8),
            chain_id: Uint256::from(1u8),
            token: "0x0000000000000000000000000000000000000000".to_string(),
            amount: Uint256::from(token1_amount),
            sender: "0x0000000000000000000000000000000000000001".to_string(),
            receiver: Addr::unchecked("liquidity_adder"),
        },
    )?;

    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        ExecuteMsg::AddLiquidity {
            pool_id: Uint256::from(0u8),
            chain_id: Uint256::from(0u8),
            token: "0x0000000000000000000000000000000000000000".to_string(),
            amount: Uint256::from(token0_amount),
            sender: "0x0000000000000000000000000000000000000001".to_string(),
            receiver: Addr::unchecked("liquidity_adder"),
        },
    )?;
    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        ExecuteMsg::AddLiquidity {
            pool_id: Uint256::from(0u8),
            chain_id: Uint256::from(1u8),
            token: "0x0000000000000000000000000000000000000000".to_string(),
            amount: Uint256::from(token0_amount),
            sender: "0x0000000000000000000000000000000000000001".to_string(),
            receiver: Addr::unchecked("liquidity_adder"),
        },
    )?;
    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::LiquidityQueue {
            pool_id: Default::default(),
        },
    )?;
    let queue_id: QueueID = from_binary(&res).unwrap();
    assert_eq!(queue_id.start, 0);
    assert_eq!(queue_id.length, 1);

    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::LiquidityQueueElement {
            pool_id: Default::default(),
            queue_id: 0,
        },
    )?;
    let liquidity_queue_element: LiquidityQueueElement = from_binary(&res).unwrap();
    assert_eq!(
        Uint256::from(token0_amount / 2),
        liquidity_queue_element.amount
    );

    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::PoolInfo {
            pool_id: Default::default(),
        },
    )?;
    let pool_info: PoolInfo = from_binary(&res).unwrap();

    assert_eq!(
        Uint256::from(
            (token0_amount * token1_amount).isqrt() + (token0_amount * token1_amount).isqrt() / 2
        ),
        pool_info.total_liquidity
    );
    assert_eq!(pool_info.fee, 3000);
    assert_eq!(
        pool_info.amount0,
        Uint256::from(token0_amount + token0_amount / 2)
    );
    assert_eq!(
        pool_info.amount1,
        Uint256::from(token1_amount + token1_amount / 2)
    );
    assert_eq!(pool_info.pending_amount0, Uint256::from(token0_amount / 2));
    assert_eq!(pool_info.pending_amount1, Uint256::zero());
    Ok(())
}

/// Create pool chain(0)_token(base coin) - chain(1)_token(base coin)
/// add liquidity into queue different amounts
#[test]
fn add_liquidity_test_5_1() -> Result<(), ContractError> {
    let mut deps = mock_dependencies();
    let info = mock_info("admin0", &[]);
    let _ = instantiate(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        InstantiateMsg {
            event_tracker: Addr::unchecked("tracker0".to_string()),
            deadline: 1000,
        },
    )?;
    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        ExecuteMsg::RegisterJobId {
            chain_id: Uint256::from(0u8),
            job: "create_pool".to_string(),
            job_id: "create_pool_eth".to_string(),
        },
    )?;
    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        ExecuteMsg::RegisterJobId {
            chain_id: Uint256::from(1u8),
            job: "create_pool".to_string(),
            job_id: "create_pool_bnb".to_string(),
        },
    )?;
    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        ExecuteMsg::CreatePool {
            chain0_id: Uint256::from(0u8),
            chain1_id: Uint256::from(1u8),
            token0: "0x0000000000000000000000000000000000000000".to_string(),
            token1: "0x0000000000000000000000000000000000000000".to_string(),
            chain0_init_depositor: "0x0000000000000000000000000000000000000001".to_string(),
            chain1_init_depositor: "0x0000000000000000000000000000000000000001".to_string(),
            fee: 3000,
        },
    )?;
    let info = mock_info("tracker0", &[]);
    let token0_amount = 1_000_000_000_000_000_000u128;
    let token1_amount = 2_000_000_000_000_000_000u128;
    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        ExecuteMsg::AddLiquidity {
            pool_id: Uint256::from(0u8),
            chain_id: Uint256::from(0u8),
            token: "0x0000000000000000000000000000000000000000".to_string(),
            amount: Uint256::from(token0_amount),
            sender: "0x0000000000000000000000000000000000000001".to_string(),
            receiver: Addr::unchecked("liquidity_adder"),
        },
    )?;
    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        ExecuteMsg::AddLiquidity {
            pool_id: Uint256::from(0u8),
            chain_id: Uint256::from(1u8),
            token: "0x0000000000000000000000000000000000000000".to_string(),
            amount: Uint256::from(token1_amount),
            sender: "0x0000000000000000000000000000000000000001".to_string(),
            receiver: Addr::unchecked("liquidity_adder"),
        },
    )?;

    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        ExecuteMsg::AddLiquidity {
            pool_id: Uint256::from(0u8),
            chain_id: Uint256::from(1u8),
            token: "0x0000000000000000000000000000000000000000".to_string(),
            amount: Uint256::from(token0_amount),
            sender: "0x0000000000000000000000000000000000000001".to_string(),
            receiver: Addr::unchecked("liquidity_adder"),
        },
    )?;
    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        ExecuteMsg::AddLiquidity {
            pool_id: Uint256::from(0u8),
            chain_id: Uint256::from(0u8),
            token: "0x0000000000000000000000000000000000000000".to_string(),
            amount: Uint256::from(token0_amount),
            sender: "0x0000000000000000000000000000000000000001".to_string(),
            receiver: Addr::unchecked("liquidity_adder"),
        },
    )?;
    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::LiquidityQueue {
            pool_id: Default::default(),
        },
    )?;
    let queue_id: QueueID = from_binary(&res).unwrap();
    assert_eq!(queue_id.start, 0);
    assert_eq!(queue_id.length, 1);

    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::LiquidityQueueElement {
            pool_id: Default::default(),
            queue_id: 0,
        },
    )?;
    let liquidity_queue_element: LiquidityQueueElement = from_binary(&res).unwrap();
    assert_eq!(
        Uint256::from(token0_amount / 2),
        liquidity_queue_element.amount
    );

    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::PoolInfo {
            pool_id: Default::default(),
        },
    )?;
    let pool_info: PoolInfo = from_binary(&res).unwrap();

    assert_eq!(
        Uint256::from(
            (token0_amount * token1_amount).isqrt() + (token0_amount * token1_amount).isqrt() / 2
        ),
        pool_info.total_liquidity
    );
    assert_eq!(pool_info.fee, 3000);
    assert_eq!(
        pool_info.amount0,
        Uint256::from(token0_amount + token0_amount / 2)
    );
    assert_eq!(
        pool_info.amount1,
        Uint256::from(token1_amount + token1_amount / 2)
    );
    assert_eq!(pool_info.pending_amount0, Uint256::from(token0_amount / 2));
    assert_eq!(pool_info.pending_amount1, Uint256::zero());
    Ok(())
}

/// Create pool chain(0)_token(base coin) - chain(1)_token(base coin)
/// add liquidity into queue different amounts
#[test]
fn add_liquidity_test_6() -> Result<(), ContractError> {
    let mut deps = mock_dependencies();
    let info = mock_info("admin0", &[]);
    let _ = instantiate(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        InstantiateMsg {
            event_tracker: Addr::unchecked("tracker0".to_string()),
            deadline: 1000,
        },
    )?;
    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        ExecuteMsg::RegisterJobId {
            chain_id: Uint256::from(0u8),
            job: "create_pool".to_string(),
            job_id: "create_pool_eth".to_string(),
        },
    )?;
    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        ExecuteMsg::RegisterJobId {
            chain_id: Uint256::from(1u8),
            job: "create_pool".to_string(),
            job_id: "create_pool_bnb".to_string(),
        },
    )?;
    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        ExecuteMsg::CreatePool {
            chain0_id: Uint256::from(0u8),
            chain1_id: Uint256::from(1u8),
            token0: "0x0000000000000000000000000000000000000000".to_string(),
            token1: "0x0000000000000000000000000000000000000000".to_string(),
            chain0_init_depositor: "0x0000000000000000000000000000000000000001".to_string(),
            chain1_init_depositor: "0x0000000000000000000000000000000000000001".to_string(),
            fee: 3000,
        },
    )?;
    let info = mock_info("tracker0", &[]);
    let token0_amount = 1_000_000_000_000_000_000u128;
    let token1_amount = 2_000_000_000_000_000_000u128;
    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        ExecuteMsg::AddLiquidity {
            pool_id: Uint256::from(0u8),
            chain_id: Uint256::from(0u8),
            token: "0x0000000000000000000000000000000000000000".to_string(),
            amount: Uint256::from(token0_amount),
            sender: "0x0000000000000000000000000000000000000001".to_string(),
            receiver: Addr::unchecked("liquidity_adder"),
        },
    )?;
    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        ExecuteMsg::AddLiquidity {
            pool_id: Uint256::from(0u8),
            chain_id: Uint256::from(1u8),
            token: "0x0000000000000000000000000000000000000000".to_string(),
            amount: Uint256::from(token1_amount),
            sender: "0x0000000000000000000000000000000000000001".to_string(),
            receiver: Addr::unchecked("liquidity_adder"),
        },
    )?;

    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        ExecuteMsg::AddLiquidity {
            pool_id: Uint256::from(0u8),
            chain_id: Uint256::from(0u8),
            token: "0x0000000000000000000000000000000000000000".to_string(),
            amount: Uint256::from(token0_amount),
            sender: "0x0000000000000000000000000000000000000001".to_string(),
            receiver: Addr::unchecked("liquidity_adder"),
        },
    )?;
    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        ExecuteMsg::AddLiquidity {
            pool_id: Uint256::from(0u8),
            chain_id: Uint256::from(1u8),
            token: "0x0000000000000000000000000000000000000000".to_string(),
            amount: Uint256::from(token1_amount * 2),
            sender: "0x0000000000000000000000000000000000000001".to_string(),
            receiver: Addr::unchecked("liquidity_adder"),
        },
    )?;
    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::LiquidityQueue {
            pool_id: Default::default(),
        },
    )?;
    let queue_id: QueueID = from_binary(&res).unwrap();
    assert_eq!(queue_id.start, 0);
    assert_eq!(queue_id.length, 1);

    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::LiquidityQueueElement {
            pool_id: Default::default(),
            queue_id: 0,
        },
    )?;
    let liquidity_queue_element: LiquidityQueueElement = from_binary(&res).unwrap();

    assert_eq!(Uint256::from(token1_amount), liquidity_queue_element.amount);

    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::PoolInfo {
            pool_id: Default::default(),
        },
    )?;
    let pool_info: PoolInfo = from_binary(&res).unwrap();
    assert_eq!(
        Uint256::from((token0_amount * token1_amount).isqrt() * 2),
        pool_info.total_liquidity
    );
    assert_eq!(3000, pool_info.fee);
    assert_eq!(Uint256::from(token0_amount * 2), pool_info.amount0);
    assert_eq!(Uint256::from(token1_amount * 2), pool_info.amount1);
    assert_eq!(Uint256::zero(), pool_info.pending_amount0);
    assert_eq!(Uint256::from(token1_amount), pool_info.pending_amount1);

    Ok(())
}

/// Create pool chain(0)_token(base coin) - chain(1)_token(base coin)
/// add liquidity into queue different amounts
#[test]
fn add_liquidity_test_6_1() -> Result<(), ContractError> {
    let mut deps = mock_dependencies();
    let info = mock_info("admin0", &[]);
    let _ = instantiate(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        InstantiateMsg {
            event_tracker: Addr::unchecked("tracker0".to_string()),
            deadline: 1000,
        },
    )?;
    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        ExecuteMsg::RegisterJobId {
            chain_id: Uint256::from(0u8),
            job: "create_pool".to_string(),
            job_id: "create_pool_eth".to_string(),
        },
    )?;
    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        ExecuteMsg::RegisterJobId {
            chain_id: Uint256::from(1u8),
            job: "create_pool".to_string(),
            job_id: "create_pool_bnb".to_string(),
        },
    )?;
    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        ExecuteMsg::CreatePool {
            chain0_id: Uint256::from(0u8),
            chain1_id: Uint256::from(1u8),
            token0: "0x0000000000000000000000000000000000000000".to_string(),
            token1: "0x0000000000000000000000000000000000000000".to_string(),
            chain0_init_depositor: "0x0000000000000000000000000000000000000001".to_string(),
            chain1_init_depositor: "0x0000000000000000000000000000000000000001".to_string(),
            fee: 3000,
        },
    )?;
    let info = mock_info("tracker0", &[]);
    let token0_amount = 1_000_000_000_000_000_000u128;
    let token1_amount = 2_000_000_000_000_000_000u128;
    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        ExecuteMsg::AddLiquidity {
            pool_id: Uint256::from(0u8),
            chain_id: Uint256::from(0u8),
            token: "0x0000000000000000000000000000000000000000".to_string(),
            amount: Uint256::from(token0_amount),
            sender: "0x0000000000000000000000000000000000000001".to_string(),
            receiver: Addr::unchecked("liquidity_adder"),
        },
    )?;
    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        ExecuteMsg::AddLiquidity {
            pool_id: Uint256::from(0u8),
            chain_id: Uint256::from(1u8),
            token: "0x0000000000000000000000000000000000000000".to_string(),
            amount: Uint256::from(token1_amount),
            sender: "0x0000000000000000000000000000000000000001".to_string(),
            receiver: Addr::unchecked("liquidity_adder"),
        },
    )?;

    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        ExecuteMsg::AddLiquidity {
            pool_id: Uint256::from(0u8),
            chain_id: Uint256::from(1u8),
            token: "0x0000000000000000000000000000000000000000".to_string(),
            amount: Uint256::from(token1_amount * 2),
            sender: "0x0000000000000000000000000000000000000001".to_string(),
            receiver: Addr::unchecked("liquidity_adder"),
        },
    )?;
    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        ExecuteMsg::AddLiquidity {
            pool_id: Uint256::from(0u8),
            chain_id: Uint256::from(0u8),
            token: "0x0000000000000000000000000000000000000000".to_string(),
            amount: Uint256::from(token0_amount),
            sender: "0x0000000000000000000000000000000000000001".to_string(),
            receiver: Addr::unchecked("liquidity_adder"),
        },
    )?;

    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::LiquidityQueue {
            pool_id: Default::default(),
        },
    )?;
    let queue_id: QueueID = from_binary(&res).unwrap();
    assert_eq!(queue_id.start, 0);
    assert_eq!(queue_id.length, 1);

    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::LiquidityQueueElement {
            pool_id: Default::default(),
            queue_id: 0,
        },
    )?;
    let liquidity_queue_element: LiquidityQueueElement = from_binary(&res).unwrap();

    assert_eq!(Uint256::from(token1_amount), liquidity_queue_element.amount);

    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::PoolInfo {
            pool_id: Default::default(),
        },
    )?;
    let pool_info: PoolInfo = from_binary(&res).unwrap();
    assert_eq!(
        Uint256::from((token0_amount * token1_amount).isqrt() * 2),
        pool_info.total_liquidity
    );
    assert_eq!(3000, pool_info.fee);
    assert_eq!(Uint256::from(token0_amount * 2), pool_info.amount0);
    assert_eq!(Uint256::from(token1_amount * 2), pool_info.amount1);
    assert_eq!(Uint256::zero(), pool_info.pending_amount0);
    assert_eq!(Uint256::from(token1_amount), pool_info.pending_amount1);

    Ok(())
}

/// Create pool chain(0)_token(base coin) - chain(1)_token(base coin)
/// add liquidity into queue different amounts
#[test]
fn add_liquidity_test_6_2() -> Result<(), ContractError> {
    let mut deps = mock_dependencies();
    let info = mock_info("admin0", &[]);
    let _ = instantiate(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        InstantiateMsg {
            event_tracker: Addr::unchecked("tracker0".to_string()),
            deadline: 1000,
        },
    )?;
    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        ExecuteMsg::RegisterJobId {
            chain_id: Uint256::from(0u8),
            job: "create_pool".to_string(),
            job_id: "create_pool_eth".to_string(),
        },
    )?;
    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        ExecuteMsg::RegisterJobId {
            chain_id: Uint256::from(1u8),
            job: "create_pool".to_string(),
            job_id: "create_pool_bnb".to_string(),
        },
    )?;
    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        ExecuteMsg::CreatePool {
            chain0_id: Uint256::from(0u8),
            chain1_id: Uint256::from(1u8),
            token0: "0x0000000000000000000000000000000000000000".to_string(),
            token1: "0x0000000000000000000000000000000000000000".to_string(),
            chain0_init_depositor: "0x0000000000000000000000000000000000000001".to_string(),
            chain1_init_depositor: "0x0000000000000000000000000000000000000001".to_string(),
            fee: 3000,
        },
    )?;
    let info = mock_info("tracker0", &[]);
    let token0_amount = 1_000_000_000_000_000_000u128;
    let token1_amount = 2_000_000_000_000_000_000u128;
    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        ExecuteMsg::AddLiquidity {
            pool_id: Uint256::from(0u8),
            chain_id: Uint256::from(0u8),
            token: "0x0000000000000000000000000000000000000000".to_string(),
            amount: Uint256::from(token0_amount),
            sender: "0x0000000000000000000000000000000000000001".to_string(),
            receiver: Addr::unchecked("liquidity_adder"),
        },
    )?;
    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        ExecuteMsg::AddLiquidity {
            pool_id: Uint256::from(0u8),
            chain_id: Uint256::from(1u8),
            token: "0x0000000000000000000000000000000000000000".to_string(),
            amount: Uint256::from(token1_amount),
            sender: "0x0000000000000000000000000000000000000001".to_string(),
            receiver: Addr::unchecked("liquidity_adder"),
        },
    )?;

    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        ExecuteMsg::AddLiquidity {
            pool_id: Uint256::from(0u8),
            chain_id: Uint256::from(1u8),
            token: "0x0000000000000000000000000000000000000000".to_string(),
            amount: Uint256::from(token1_amount * 2),
            sender: "0x0000000000000000000000000000000000000001".to_string(),
            receiver: Addr::unchecked("liquidity_adder"),
        },
    )?;
    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        ExecuteMsg::AddLiquidity {
            pool_id: Uint256::from(0u8),
            chain_id: Uint256::from(0u8),
            token: "0x0000000000000000000000000000000000000000".to_string(),
            amount: Uint256::from(token0_amount),
            sender: "0x0000000000000000000000000000000000000001".to_string(),
            receiver: Addr::unchecked("liquidity_adder"),
        },
    )?;

    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::LiquidityQueue {
            pool_id: Default::default(),
        },
    )?;
    let queue_id: QueueID = from_binary(&res).unwrap();
    assert_eq!(queue_id.start, 0);
    assert_eq!(queue_id.length, 1);

    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::LiquidityQueueElement {
            pool_id: Default::default(),
            queue_id: 0,
        },
    )?;
    let liquidity_queue_element: LiquidityQueueElement = from_binary(&res).unwrap();

    assert_eq!(Uint256::from(token1_amount), liquidity_queue_element.amount);

    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::PoolInfo {
            pool_id: Default::default(),
        },
    )?;
    let pool_info: PoolInfo = from_binary(&res).unwrap();
    assert_eq!(
        Uint256::from((token0_amount * token1_amount).isqrt() * 2),
        pool_info.total_liquidity
    );
    assert_eq!(3000, pool_info.fee);
    assert_eq!(Uint256::from(token0_amount * 2), pool_info.amount0);
    assert_eq!(Uint256::from(token1_amount * 2), pool_info.amount1);
    assert_eq!(Uint256::zero(), pool_info.pending_amount0);
    assert_eq!(Uint256::from(token1_amount), pool_info.pending_amount1);

    Ok(())
}

#[test]
fn remove_liquidity_test() -> Result<(), ContractError> {
    let mut deps = mock_dependencies();
    let info = mock_info("admin0", &[]);
    let _ = instantiate(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        InstantiateMsg {
            event_tracker: Addr::unchecked("tracker0".to_string()),
            deadline: 1000,
        },
    )?;
    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        ExecuteMsg::RegisterJobId {
            chain_id: Uint256::from(0u8),
            job: "create_pool".to_string(),
            job_id: "create_pool_eth".to_string(),
        },
    )?;
    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        ExecuteMsg::RegisterJobId {
            chain_id: Uint256::from(1u8),
            job: "create_pool".to_string(),
            job_id: "create_pool_bnb".to_string(),
        },
    )?;
    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        ExecuteMsg::RegisterJobId {
            chain_id: Uint256::from(0u8),
            job: "remove_liquidity".to_string(),
            job_id: "remove_liquidity".to_string(),
        },
    )?;
    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        ExecuteMsg::RegisterJobId {
            chain_id: Uint256::from(1u8),
            job: "remove_liquidity".to_string(),
            job_id: "remove_liquidity".to_string(),
        },
    )?;
    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        ExecuteMsg::CreatePool {
            chain0_id: Uint256::from(0u8),
            chain1_id: Uint256::from(1u8),
            token0: "0x0000000000000000000000000000000000000000".to_string(),
            token1: "0x1234567890123456789012345678901234567890".to_string(),
            chain0_init_depositor: "0x0000000000000000000000000000000000000001".to_string(),
            chain1_init_depositor: "0x0000000000000000000000000000000000000001".to_string(),
            fee: 3000,
        },
    )?;
    let info = mock_info("tracker0", &[]);
    let token_amount = 1_000_000_000_000_000_000u128;
    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        ExecuteMsg::AddLiquidity {
            pool_id: Uint256::from(0u8),
            chain_id: Uint256::from(0u8),
            token: "0x0000000000000000000000000000000000000000".to_string(),
            amount: Uint256::from(token_amount),
            sender: "0x0000000000000000000000000000000000000001".to_string(),
            receiver: Addr::unchecked("liquidity_adder"),
        },
    )?;
    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        ExecuteMsg::AddLiquidity {
            pool_id: Uint256::from(0u8),
            chain_id: Uint256::from(1u8),
            token: "0x1234567890123456789012345678901234567890".to_string(),
            amount: Uint256::from(token_amount),
            sender: "0x0000000000000000000000000000000000000001".to_string(),
            receiver: Addr::unchecked("liquidity_adder"),
        },
    )?;
    let info = mock_info("liquidity_adder", &[]);
    let res = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        ExecuteMsg::RemoveLiquidity {
            chain0_id: Uint256::from(0u8),
            chain1_id: Uint256::from(1u8),
            token0: "0x0000000000000000000000000000000000000000".to_string(),
            token1: "0x1234567890123456789012345678901234567890".to_string(),
            receiver0: "0x1000000000000000000000000000000000000000".to_string(),
            receiver1: "0x1000000000000000000000000000000000000000".to_string(),
            amount: Uint256::from(token_amount / 2),
        },
    )?;
    assert_eq!(res.messages.len(), 2);
    Ok(())
}

#[test]
fn remove_liquidity_test_1() -> Result<(), ContractError> {
    let mut deps = mock_dependencies();
    let info = mock_info("admin0", &[]);
    let _ = instantiate(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        InstantiateMsg {
            event_tracker: Addr::unchecked("tracker0".to_string()),
            deadline: 1000,
        },
    )?;
    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        ExecuteMsg::RegisterJobId {
            chain_id: Uint256::from(0u8),
            job: "create_pool".to_string(),
            job_id: "create_pool_eth".to_string(),
        },
    )?;
    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        ExecuteMsg::RegisterJobId {
            chain_id: Uint256::from(1u8),
            job: "create_pool".to_string(),
            job_id: "create_pool_bnb".to_string(),
        },
    )?;
    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        ExecuteMsg::RegisterJobId {
            chain_id: Uint256::from(0u8),
            job: "remove_liquidity".to_string(),
            job_id: "remove_liquidity".to_string(),
        },
    )?;
    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        ExecuteMsg::RegisterJobId {
            chain_id: Uint256::from(1u8),
            job: "remove_liquidity".to_string(),
            job_id: "remove_liquidity".to_string(),
        },
    )?;
    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        ExecuteMsg::CreatePool {
            chain0_id: Uint256::from(0u8),
            chain1_id: Uint256::from(0u8),
            token0: "0x1234567890123456789012345678901234567890".to_string(),
            token1: "0x0000000000000000000000000000000000000000".to_string(),
            chain0_init_depositor: "0x0000000000000000000000000000000000000001".to_string(),
            chain1_init_depositor: "0x0000000000000000000000000000000000000001".to_string(),
            fee: 3000,
        },
    )?;
    let info = mock_info("tracker0", &[]);
    let token_amount = 1_000_000_000_000_000_000u128;
    let pool_id: Uint256 = from_binary(&query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::PoolId {
            chain0_id: Uint256::from(0u8),
            chain1_id: Uint256::from(0u8),
            token0: "0x0000000000000000000000000000000000000000".to_string(),
            token1: "0x1234567890123456789012345678901234567890".to_string(),
        },
    )?)
    .unwrap();
    let pool_id_1: Uint256 = from_binary(&query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::PoolId {
            chain0_id: Uint256::from(0u8),
            chain1_id: Uint256::from(0u8),
            token0: "0x1234567890123456789012345678901234567890".to_string(),
            token1: "0x0000000000000000000000000000000000000000".to_string(),
        },
    )?)
    .unwrap();
    assert_eq!(pool_id, pool_id_1);
    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        ExecuteMsg::AddLiquidity {
            pool_id,
            chain_id: Uint256::from(0u8),
            token: "0x1234567890123456789012345678901234567890".to_string(),
            amount: Uint256::from(token_amount),
            sender: "0x0000000000000000000000000000000000000001".to_string(),
            receiver: Addr::unchecked("liquidity_adder"),
        },
    )?;
    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        ExecuteMsg::AddLiquidity {
            pool_id: Uint256::from(0u8),
            chain_id: Uint256::from(0u8),
            token: "0x0000000000000000000000000000000000000000".to_string(),
            amount: Uint256::from(token_amount),
            sender: "0x0000000000000000000000000000000000000001".to_string(),
            receiver: Addr::unchecked("liquidity_adder"),
        },
    )?;
    let info = mock_info("liquidity_adder", &[]);
    let res = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        ExecuteMsg::RemoveLiquidity {
            chain0_id: Uint256::from(0u8),
            chain1_id: Uint256::from(0u8),
            token0: "0x1234567890123456789012345678901234567890".to_string(),
            token1: "0x0000000000000000000000000000000000000000".to_string(),
            receiver0: "0x1000000000000000000000000000000000000000".to_string(),
            receiver1: "0x1000000000000000000000000000000000000000".to_string(),
            amount: Uint256::from(token_amount / 2),
        },
    )?;
    assert_eq!(res.messages.len(), 2);
    Ok(())
}

#[test]
fn swap_test_0() -> Result<(), ContractError> {
    let mut deps = mock_dependencies();
    let info = mock_info("admin0", &[]);
    let _ = instantiate(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        InstantiateMsg {
            event_tracker: Addr::unchecked("tracker0".to_string()),
            deadline: 1000,
        },
    )?;
    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        ExecuteMsg::RegisterJobId {
            chain_id: Uint256::from(0u8),
            job: "create_pool".to_string(),
            job_id: "create_pool_eth".to_string(),
        },
    )?;
    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        ExecuteMsg::RegisterJobId {
            chain_id: Uint256::from(1u8),
            job: "create_pool".to_string(),
            job_id: "create_pool_bnb".to_string(),
        },
    )?;
    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        ExecuteMsg::RegisterJobId {
            chain_id: Uint256::from(0u8),
            job: "swap_out".to_string(),
            job_id: "swap_out".to_string(),
        },
    )?;
    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        ExecuteMsg::RegisterJobId {
            chain_id: Uint256::from(1u8),
            job: "swap_out".to_string(),
            job_id: "swap_out".to_string(),
        },
    )?;
    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        ExecuteMsg::RegisterJobId {
            chain_id: Uint256::from(0u8),
            job: "withdraw_fee".to_string(),
            job_id: "withdraw_fee".to_string(),
        },
    )?;
    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        ExecuteMsg::RegisterJobId {
            chain_id: Uint256::from(1u8),
            job: "withdraw_fee".to_string(),
            job_id: "withdraw_fee".to_string(),
        },
    )?;
    let pool_fee = 30u16;
    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        ExecuteMsg::CreatePool {
            chain0_id: Uint256::from(0u8),
            chain1_id: Uint256::from(1u8),
            token0: "0x0000000000000000000000000000000000000000".to_string(),
            token1: "0x1234567890123456789012345678901234567890".to_string(),
            chain0_init_depositor: "0x0000000000000000000000000000000000000001".to_string(),
            chain1_init_depositor: "0x0000000000000000000000000000000000000001".to_string(),
            fee: pool_fee,
        },
    )?;
    let info = mock_info("tracker0", &[]);
    let token_amount = 1_000_000_000_000_000_000u128;
    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        ExecuteMsg::AddLiquidity {
            pool_id: Uint256::from(0u8),
            chain_id: Uint256::from(0u8),
            token: "0x0000000000000000000000000000000000000000".to_string(),
            amount: Uint256::from(token_amount),
            sender: "0x0000000000000000000000000000000000000001".to_string(),
            receiver: Addr::unchecked("liquidity_adder"),
        },
    )?;
    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        ExecuteMsg::AddLiquidity {
            pool_id: Uint256::from(0u8),
            chain_id: Uint256::from(1u8),
            token: "0x1234567890123456789012345678901234567890".to_string(),
            amount: Uint256::from(token_amount),
            sender: "0x0000000000000000000000000000000000000001".to_string(),
            receiver: Addr::unchecked("liquidity_adder"),
        },
    )?;
    let res = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        ExecuteMsg::Swap {
            pool_id: Default::default(),
            chain_from_id: Uint256::from(0u8),
            token_from: "0x0000000000000000000000000000000000000000".to_string(),
            amount: Uint256::from(token_amount),
            receiver: "0x1000000000000000000000000000000000000000".to_string(),
        },
    )?;
    assert_eq!(res.messages.len(), 2);
    let mut iter = res.messages.iter();
    if let Some(msg) = iter.next() {
        if let CosmosMsg::Custom(data) = msg.msg.clone() {
            assert_eq!("swap_out".to_string(), data.job_id);
        }
    }
    if let Some(msg) = iter.next() {
        if let CosmosMsg::Custom(data) = msg.msg.clone() {
            assert_eq!("withdraw_fee".to_string(), data.job_id);
        }
    }
    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::PoolInfo {
            pool_id: Default::default(),
        },
    )?;
    let pool_info: PoolInfo = from_binary(&res).unwrap();
    let management_fee = 3000u16;
    let denominator = 10000u16;
    let added_amount = Uint256::try_from(
        Uint512::from(token_amount)
            - (Uint512::from(token_amount)
                * Uint512::from(management_fee)
                * Uint512::from(pool_fee)
                / Uint512::from(denominator)
                / Uint512::from(denominator)),
    )
    .unwrap();
    assert_eq!(
        pool_info.amount0,
        Uint256::from(token_amount) + added_amount
    );
    let subbed_amount = Uint256::try_from(
        Uint512::from(added_amount)
            * Uint512::from(token_amount)
            * Uint512::from(denominator - pool_fee)
            / (Uint512::from(added_amount) * Uint512::from(denominator - pool_fee)
                + Uint512::from(token_amount) * Uint512::from(denominator)),
    )
    .unwrap();
    assert_eq!(
        pool_info.amount1,
        Uint256::from(token_amount) - subbed_amount
    );
    assert_eq!(Uint256::zero(), pool_info.pending_amount0);
    assert_eq!(Uint256::zero(), pool_info.pending_amount1);
    assert_eq!(Uint256::from(token_amount), pool_info.total_liquidity);
    Ok(())
}

#[test]
fn swap_test_1() -> Result<(), ContractError> {
    let mut deps = mock_dependencies();
    let info = mock_info("admin0", &[]);
    let _ = instantiate(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        InstantiateMsg {
            event_tracker: Addr::unchecked("tracker0".to_string()),
            deadline: 1000,
        },
    )?;
    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        ExecuteMsg::RegisterJobId {
            chain_id: Uint256::from(0u8),
            job: "create_pool".to_string(),
            job_id: "create_pool_eth".to_string(),
        },
    )?;
    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        ExecuteMsg::RegisterJobId {
            chain_id: Uint256::from(1u8),
            job: "create_pool".to_string(),
            job_id: "create_pool_bnb".to_string(),
        },
    )?;
    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        ExecuteMsg::RegisterJobId {
            chain_id: Uint256::from(0u8),
            job: "swap_out".to_string(),
            job_id: "swap_out".to_string(),
        },
    )?;
    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        ExecuteMsg::RegisterJobId {
            chain_id: Uint256::from(1u8),
            job: "swap_out".to_string(),
            job_id: "swap_out".to_string(),
        },
    )?;
    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        ExecuteMsg::RegisterJobId {
            chain_id: Uint256::from(0u8),
            job: "withdraw_fee".to_string(),
            job_id: "withdraw_fee".to_string(),
        },
    )?;
    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        ExecuteMsg::RegisterJobId {
            chain_id: Uint256::from(1u8),
            job: "withdraw_fee".to_string(),
            job_id: "withdraw_fee".to_string(),
        },
    )?;
    let pool_fee = 30u16;
    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        ExecuteMsg::CreatePool {
            chain0_id: Uint256::from(0u8),
            chain1_id: Uint256::from(1u8),
            token0: "0x0000000000000000000000000000000000000000".to_string(),
            token1: "0x1234567890123456789012345678901234567890".to_string(),
            chain0_init_depositor: "0x0000000000000000000000000000000000000001".to_string(),
            chain1_init_depositor: "0x0000000000000000000000000000000000000001".to_string(),
            fee: pool_fee,
        },
    )?;
    let info = mock_info("tracker0", &[]);
    let token_amount = 1_000_000_000_000_000_000u128;
    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        ExecuteMsg::AddLiquidity {
            pool_id: Uint256::from(0u8),
            chain_id: Uint256::from(0u8),
            token: "0x0000000000000000000000000000000000000000".to_string(),
            amount: Uint256::from(token_amount),
            sender: "0x0000000000000000000000000000000000000001".to_string(),
            receiver: Addr::unchecked("liquidity_adder"),
        },
    )?;
    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        ExecuteMsg::AddLiquidity {
            pool_id: Uint256::from(0u8),
            chain_id: Uint256::from(1u8),
            token: "0x1234567890123456789012345678901234567890".to_string(),
            amount: Uint256::from(token_amount),
            sender: "0x0000000000000000000000000000000000000001".to_string(),
            receiver: Addr::unchecked("liquidity_adder"),
        },
    )?;
    let res = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        ExecuteMsg::Swap {
            pool_id: Default::default(),
            chain_from_id: Uint256::from(1u8),
            token_from: "0x0000000000000000000000000000000000000000".to_string(),
            amount: Uint256::from(token_amount),
            receiver: "0x1000000000000000000000000000000000000000".to_string(),
        },
    )?;
    assert_eq!(res.messages.len(), 2);
    let mut iter = res.messages.iter();
    if let Some(msg) = iter.next() {
        if let CosmosMsg::Custom(data) = msg.msg.clone() {
            assert_eq!("swap_out".to_string(), data.job_id);
        }
    }
    if let Some(msg) = iter.next() {
        if let CosmosMsg::Custom(data) = msg.msg.clone() {
            assert_eq!("withdraw_fee".to_string(), data.job_id);
        }
    }
    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::PoolInfo {
            pool_id: Default::default(),
        },
    )?;
    let pool_info: PoolInfo = from_binary(&res).unwrap();
    let management_fee = 3000u16;
    let denominator = 10000u16;
    let added_amount = Uint256::try_from(
        Uint512::from(token_amount)
            - (Uint512::from(token_amount)
                * Uint512::from(management_fee)
                * Uint512::from(pool_fee)
                / Uint512::from(denominator)
                / Uint512::from(denominator)),
    )
    .unwrap();
    let subbed_amount = Uint256::try_from(
        Uint512::from(added_amount)
            * Uint512::from(token_amount)
            * Uint512::from(denominator - pool_fee)
            / (Uint512::from(added_amount) * Uint512::from(denominator - pool_fee)
                + Uint512::from(token_amount) * Uint512::from(denominator)),
    )
    .unwrap();
    assert_eq!(
        pool_info.amount0,
        Uint256::from(token_amount) - subbed_amount
    );
    assert_eq!(
        pool_info.amount1,
        Uint256::from(token_amount) + added_amount
    );
    assert_eq!(Uint256::zero(), pool_info.pending_amount0);
    assert_eq!(Uint256::zero(), pool_info.pending_amount1);
    assert_eq!(Uint256::from(token_amount), pool_info.total_liquidity);
    Ok(())
}

#[test]
fn update_config_test() -> Result<(), ContractError> {
    let mut deps = mock_dependencies();
    let info = mock_info("admin0", &[]);
    let _ = instantiate(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        InstantiateMsg {
            event_tracker: Addr::unchecked("tracker0".to_string()),
            deadline: 1000,
        },
    )?;
    let new_deadline = 2000u64;
    let new_fee = 2500u16;
    let new_admin = Addr::unchecked("admin1".to_string());
    let new_event_tracker = Addr::unchecked("tracker1".to_string());

    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        ExecuteMsg::UpdateConfig {
            new_deadline: Some(new_deadline),
            new_fee: Some(new_fee),
            new_admin: Some(new_admin.clone()),
            new_event_tracker: Some(new_event_tracker.clone()),
        },
    )?;
    let res = query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap();
    let state: State = from_binary(&res).unwrap();
    assert_eq!(state.deadline, new_deadline);
    assert_eq!(state.fee, new_fee);
    assert_eq!(state.admin, new_admin);
    assert_eq!(state.event_tracker, new_event_tracker);
    Ok(())
}
