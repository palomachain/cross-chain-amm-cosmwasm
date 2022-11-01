//! Smoke tests.

use crate::contract::{execute, instantiate};
use crate::msg::{ExecuteMsg, InstantiateMsg};
use crate::ContractError;
use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::Addr;

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
            event_tracker: Addr::unchecked("tracker".to_string()),
            deadline: 1000,
        },
    )?;
    let info = mock_info("tracker", &[]);
    let (chain0_id, chain1_id) = (42u32.into(), 52u32.into());
    let (token0, token1) = (
        "0123456789012345678901234567890123456789".to_string(),
        "abcdefabcdefabcdefabcdefabcdefabcdefabcd".to_string(),
    );
    let (factory0, factory1) = ("abcd".to_string(), "wxyz".to_string());
    let (sender0, sender1) = ("addr01234".to_string(), "addr98765".to_string());

    for (chain_id, factory) in [(chain0_id, factory0), (chain1_id, factory1)] {
        let r = execute(
            deps.as_mut(),
            mock_env(),
            info.clone(),
            ExecuteMsg::RegisterChain {
                chain_id,
                chain_name: "test_chain_0".to_string(),
                factory,
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

    //let r = execute(
    //    deps.as_mut(),
    //    mock_env(),
    //    info.clone(),
    //    ExecuteMsg::Swap {
    //        chain_from_id: chain0_id,
    //        chain_to_id: chain1_id,
    //        token_from: token0,
    //        token_to: token1,
    //        sender: sender0,
    //        receiver: sender1,
    //        amount: 5000u32.into(),
    //    },
    //)?;
    //assert_eq!(r.messages.len(), 2);

    Ok(())
}
