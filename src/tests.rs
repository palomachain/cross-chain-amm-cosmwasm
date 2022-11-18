//! Smoke tests.

use crate::contract::{execute, instantiate, query};
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{LiquidityQueueElement, PoolInfo, QueueID, State};
use crate::ContractError;
use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{to_binary, Addr, Isqrt, Uint256, from_binary};

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
fn create_pool_test() -> Result<(), ContractError> {
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
            queue_id: 0
        },
    )?;
    let queue_element: LiquidityQueueElement = from_binary(&res).unwrap();
    assert_eq!(queue_element.amount, Uint256::from(token0_amount));
    assert_eq!(queue_element.receiver, Addr::unchecked("liquidity_adder"));
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
            pool_id: Default::default()
        }
    )?;
    let pool_info: PoolInfo = from_binary(&res).unwrap();
    assert_eq!(Uint256::from((token0_amount * token1_amount).isqrt() * 2), pool_info.total_liquidity);
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
            queue_id: 0
        },
    )?;
    let liquidity_queue_element: LiquidityQueueElement = from_binary(&res).unwrap();
    assert_eq!(Uint256::from(token0_amount / 2), liquidity_queue_element.amount);

    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::PoolInfo {
            pool_id: Default::default()
        }
    )?;
    let pool_info: PoolInfo = from_binary(&res).unwrap();

    assert_eq!(Uint256::from((token0_amount * token1_amount).isqrt() + (token0_amount * token1_amount).isqrt() / 2), pool_info.total_liquidity);
    assert_eq!(pool_info.fee, 3000);
    assert_eq!(pool_info.amount0, Uint256::from(token0_amount + token0_amount / 2));
    assert_eq!(pool_info.amount1, Uint256::from(token1_amount + token1_amount / 2));
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
            queue_id: 0
        },
    )?;
    let liquidity_queue_element: LiquidityQueueElement = from_binary(&res).unwrap();

    assert_eq!(Uint256::from(token1_amount), liquidity_queue_element.amount);

    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::PoolInfo {
            pool_id: Default::default()
        }
    )?;
    let pool_info: PoolInfo = from_binary(&res).unwrap();
    assert_eq!(Uint256::from((token0_amount * token1_amount).isqrt() * 2), pool_info.total_liquidity);
    assert_eq!(3000, pool_info.fee);
    assert_eq!(Uint256::from(token0_amount * 2), pool_info.amount0);
    assert_eq!(Uint256::from(token1_amount * 2), pool_info.amount1);
    assert_eq!(Uint256::zero(), pool_info.pending_amount0);
    assert_eq!(Uint256::from(token1_amount), pool_info.pending_amount1);

    Ok(())
}

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
            receiver0: "receiver0".to_string(),
            receiver1: "receiver1".to_string(),
            amount: Uint256::from(token_amount / 2),
        },
    )?;
    assert_eq!(res.messages.len(), 2);
    Ok(())
}

fn swap_test() -> Result<(), ContractError> {
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
    let res = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        ExecuteMsg::Swap {
            pool_id: Default::default(),
            chain_from_id: Uint256::from(0u8),
            token_from: "0x0000000000000000000000000000000000000000".to_string(),
            amount: Uint256::from(token_amount),
            receiver: "receiver".to_string()
        },
    )?;
    assert_eq!(res.messages.len(), 1);
    Ok(())
}
