//! actually simulates mantis
use cosmrs::tendermint::block::Height;
use cosmwasm_std::{testing::*, Addr, Binary, Coin, MessageInfo};
use cw_mantis_order::{sv::*, OrderItem, OrderSubMsg, SolutionSubMsg};
use mantis_node::{
    mantis::{simulate::randomize_order, solve::PairSolution},
    prelude::*,
};

#[test]
fn cows_single_no_match() {
    let (mut deps, env, info) = instantiate_cow_only_orders_contract();

    // order 1
    let msg = ExecMsg::Order {
        msg: OrderSubMsg {
            wants: Coin {
                denom: "b".to_string(),
                amount: 200000u128.into(),
            },
            transfer: None,
            timeout: 1,
            min_fill: None,
        },
    };
    let msg = cw_mantis_order::sv::ContractExecMsg::OrderContract(msg);
    let given = Coin::new(2, "a");
    let info = MessageInfo {
        funds: vec![given],
        sender: Addr::unchecked("sender"),
    };
    cw_mantis_order::entry_points::execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

    // try solve
    let msg = QueryMsg::GetAllOrders {};
    let msg = cw_mantis_order::sv::ContractQueryMsg::OrderContract(msg);
    let orders: Binary =
        cw_mantis_order::entry_points::query(deps.as_ref(), env.clone(), msg).unwrap();
    let orders: Vec<OrderItem> = serde_json_wasm::from_slice(orders.as_slice()).unwrap();
    let cows = mantis_node::mantis::solve::find_cows(orders.as_slice());
    assert!(cows.is_empty());
}

#[test]
fn cows_two_perfect_math_orders() {
    let (mut deps, mut env, info) = instantiate_cow_only_orders_contract();

    // order 1
    let msg = ExecMsg::Order {
        msg: OrderSubMsg {
            wants: Coin {
                denom: "b".to_string(),
                amount: 200000u128.into(),
            },
            transfer: None,
            timeout: 1,
            min_fill: None,
        },
    };
    let msg = cw_mantis_order::sv::ContractExecMsg::OrderContract(msg);
    let given = Coin::new(2, "a");
    let info = MessageInfo {
        funds: vec![given],
        sender: Addr::unchecked("sender"),
    };
    cw_mantis_order::entry_points::execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

    // order 2 perfect match
    let msg = ExecMsg::Order {
        msg: OrderSubMsg {
            wants: Coin {
                denom: "a".to_string(),
                amount: 2u128.into(),
            },
            transfer: None,
            timeout: 1,
            min_fill: None,
        },
    };
    let given = Coin::new(200000u128, "b");
    send_order(msg, given, &mut deps, &env);

    // try solve
    let orders = query_all_orders(&deps, &env);
    assert_eq!(orders.len(), 2);

    let cows_per_pair = mantis_node::mantis::solve::find_cows(orders.as_slice());
    assert_eq!(cows_per_pair.len(), 1);
    execute_solve(cows_per_pair.clone(), &mut deps, &env, info.clone());
    wait_for_blocks(cw_mantis_order::constants::DEFAULT_BATCH_EPOCH, &mut env);
    let orders = query_all_orders(&deps, &env);
    assert_eq!(orders.len(), 2);

    execute_solve(cows_per_pair, &mut deps, &env, info.clone());
    let orders = query_all_orders(&deps, &env);
    assert_eq!(orders.len(), 0);
}

fn wait_for_blocks(blocks: u32, env: &mut cosmwasm_std::Env) {
    env.block.height += blocks as u64;
}

#[test]
fn cows_scenarios() {
    let (mut deps, env, info) = instantiate_cow_only_orders_contract();

    // 2 user give more than others wants is ok

    // pair 1
    let msg = ExecMsg::Order {
        msg: OrderSubMsg {
            wants: Coin {
                denom: "a".to_string(),
                amount: 200000u128.into(),
            },
            transfer: None,
            timeout: 1,
            min_fill: None,
        },
    };
    let given = Coin::new(2u128, "b");
    send_order(msg, given, &mut deps, &env);

    let msg = ExecMsg::Order {
        msg: OrderSubMsg {
            wants: Coin {
                denom: "b".to_string(),
                amount: 2u128.into(),
            },
            transfer: None,
            timeout: 1,
            min_fill: None,
        },
    };
    let given = Coin::new(200000u128, "a");
    send_order(msg, given, &mut deps, &env);

    // pair 2
    let msg = ExecMsg::Order {
        msg: OrderSubMsg {
            wants: Coin {
                denom: "a".to_string(),
                amount: 200000u128.into(),
            },
            transfer: None,
            timeout: 1,
            min_fill: None,
        },
    };
    let given = Coin::new(2u128, "b");
    send_order(msg, given, &mut deps, &env);

    let msg = ExecMsg::Order {
        msg: OrderSubMsg {
            wants: Coin {
                denom: "b".to_string(),
                amount: 2u128.into(),
            },
            transfer: None,
            timeout: 1,
            min_fill: None,
        },
    };
    let given = Coin::new(200000u128, "a");
    send_order(msg, given, &mut deps, &env);

    // solving
    let orders = query_all_orders(&deps, &env);
    let cows_per_pair = mantis_node::mantis::solve::find_cows(orders.as_slice());
    execute_solve(cows_per_pair, &mut deps, &env, info.clone());
    let orders = query_all_orders(&deps, &env);
    assert!(orders.is_empty());

    // partial orders

    // order
    let msg = ExecMsg::Order {
        msg: OrderSubMsg {
            wants: Coin {
                denom: "a".to_string(),
                amount: 200000u128.into(),
            },
            transfer: None,
            timeout: 1,
            min_fill: None,
        },
    };
    let given = Coin::new(2u128, "b");
    send_order(msg, given, &mut deps, &env);

    // half
    let msg = ExecMsg::Order {
        msg: OrderSubMsg {
            wants: Coin {
                denom: "b".to_string(),
                amount: (2u128 / 2).into(),
            },
            transfer: None,
            timeout: 1,
            min_fill: None,
        },
    };
    let given = Coin::new(200000u128 / 2, "a");
    send_order(msg, given, &mut deps, &env);

    // second half
    let orders = query_all_orders(&deps, &env);
    let cows_per_pair = mantis_node::mantis::solve::find_cows(orders.as_slice());
    execute_solve(cows_per_pair, &mut deps, &env, info.clone());
    let orders = query_all_orders(&deps, &env);
    assert!(!orders.is_empty());

    let msg = ExecMsg::Order {
        msg: OrderSubMsg {
            wants: Coin {
                denom: "b".to_string(),
                amount: (2u128 / 2).into(),
            },
            transfer: None,
            timeout: 1,
            min_fill: None,
        },
    };
    let given = Coin::new(200000u128 / 2, "a");
    send_order(msg, given, &mut deps, &env);

    // solving
    let orders = query_all_orders(&deps, &env);
    let cows_per_pair = mantis_node::mantis::solve::find_cows(orders.as_slice());
    execute_solve(cows_per_pair, &mut deps, &env, info.clone());
    let orders = query_all_orders(&deps, &env);
    assert!(orders.is_empty());

    for _ in 1..100 {
        let (msg, funds): (cw_mantis_order::ExecMsg, cosmrs::Coin) = randomize_order(
            &"20a,200000b".to_string(),
            Height::try_from(env.block.height).unwrap(),
        );

        send_order(
            msg,
            Coin {
                denom: funds.denom.to_string(),
                amount: funds.amount.into(),
            },
            &mut deps,
            &env,
        );
    }

    let orders = query_all_orders(&deps, &env);
    let cows_per_pair = mantis_node::mantis::solve::find_cows(orders.as_slice());
    let responses = execute_solve(cows_per_pair, &mut deps, &env, info.clone());
    let orders = query_all_orders(&deps, &env);
    println!("solved {}", orders.len());
    println!("{:?}", responses);
}

fn instantiate_cow_only_orders_contract() -> (
    cosmwasm_std::OwnedDeps<cosmwasm_std::MemoryStorage, MockApi, MockQuerier>,
    cosmwasm_std::Env,
    MessageInfo,
) {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info("sender", &[]);

    let msg = InstantiateMsg {
        admin: Some(Addr::unchecked("sender".to_string())),
        cvm_address: Addr::unchecked("cows only".to_string()),
    };
    cw_mantis_order::entry_points::instantiate(deps.as_mut(), env.clone(), info.clone(), msg)
        .unwrap();
    (deps, env, info)
}

fn execute_solve(
    cows_per_pair: Vec<PairSolution>,
    deps: &mut cosmwasm_std::OwnedDeps<cosmwasm_std::MemoryStorage, MockApi, MockQuerier>,
    env: &cosmwasm_std::Env,
    info: MessageInfo,
) -> Vec<cosmwasm_std::Response> {
    let mut responses = vec![];
    for PairSolution {
        cows,
        optimal_price,
        ..
    } in cows_per_pair
    {
        let msg = ExecMsg::Solve {
            msg: SolutionSubMsg {
                cows,
                optimal_price,
                route: None,
                timeout: 12,
            },
        };
        let msg = cw_mantis_order::sv::ContractExecMsg::OrderContract(msg);

        let response =
            cw_mantis_order::entry_points::execute(deps.as_mut(), env.clone(), info.clone(), msg)
                .expect("solve");
        responses.push(response);
    }
    responses
}

fn send_order(
    msg: ExecMsg,
    given: Coin,
    deps: &mut cosmwasm_std::OwnedDeps<cosmwasm_std::MemoryStorage, MockApi, MockQuerier>,
    env: &cosmwasm_std::Env,
) {
    let msg = cw_mantis_order::sv::ContractExecMsg::OrderContract(msg);
    let info = MessageInfo {
        funds: vec![given],
        sender: Addr::unchecked("sender"),
    };
    cw_mantis_order::entry_points::execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
}

fn query_all_orders(
    deps: &cosmwasm_std::OwnedDeps<cosmwasm_std::MemoryStorage, MockApi, MockQuerier>,
    env: &cosmwasm_std::Env,
) -> Vec<OrderItem> {
    let msg = QueryMsg::GetAllOrders {};
    let msg = cw_mantis_order::sv::ContractQueryMsg::OrderContract(msg);
    let orders: Binary =
        cw_mantis_order::entry_points::query(deps.as_ref(), env.clone(), msg).unwrap();
    serde_json_wasm::from_slice(orders.as_slice()).unwrap()
}
