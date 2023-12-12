//! actually simulates mantis
use cosmwasm_std::{testing::*, Addr, Binary, Coin, MessageInfo, DepsMut, CustomQuery};
use cw_mantis_order::{sv::*, OrderItem, OrderSubMsg, SolutionSubMsg};
use mantis_node::prelude::*;
// let msg = cvm_runtime::gateway::InstantiateMsg(HereItem {
//     network_id: 2.into(),
//     admin: Addr::unchecked("sender"),
// });
// crate::contract::instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

// let config = [];
// let msg = cvm_runtime::gateway::ExecuteMsg::Config(
//     cvm_runtime::gateway::ConfigSubMsg::Force(config.to_vec()),
// );

// crate::contract::execute::execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

// let transfer = XcInstruction::transfer_absolute_to_account("bob", 2, 100);
// let spawn = Instruction::Spawn {
//     network_id: 2.into(),
//     salt: <_>::default(),
//     assets: XcFundsFilter::one(1.into(), Amount::new(100, 0)),
//     program: XcProgram {
//         tag: <_>::default(),
//         instructions: vec![transfer],
//     },
// };
// let program = XcProgram {
//     tag: <_>::default(),
//     instructions: vec![spawn],
// };

// // crate::contract::query::query(deps.as_ref(), env.clone(), info.clone(), msg).unwrap();

#[test]
fn cows_scenarios() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info("sender", &[]);

    let msg = InstantiateMsg {
        admin: Some(Addr::unchecked("sender".to_string())),
        cvm_address: Addr::unchecked("cows only".to_string()),
    };
    cw_mantis_order::entry_points::instantiate(deps.as_mut(), env.clone(), info.clone(), msg)
        .unwrap();

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

    /// try solve
    let msg = QueryMsg::GetAllOrders {};
    let msg = cw_mantis_order::sv::ContractQueryMsg::OrderContract(msg);
    let orders: Binary =
        cw_mantis_order::entry_points::query(deps.as_ref(), env.clone(), msg).unwrap();
    let orders: Vec<OrderItem> = serde_json_wasm::from_slice(orders.as_slice()).unwrap();
    let cows = mantis_node::mantis::mantis::do_cows(orders);
    assert!(cows.is_empty());

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

    /// try solve
    let orders = query_all_orders(&deps, &env);
    let cows_per_pair = mantis_node::mantis::mantis::do_cows(orders);
    for (cows, cow_optional_price) in cows_per_pair {
        let msg = ExecMsg::Solve {
            msg: SolutionSubMsg {
                cows,
                cow_optional_price,
                route: None,
                timeout: 12,
            },
        };
        let msg = cw_mantis_order::sv::ContractExecMsg::OrderContract(msg);

        cw_mantis_order::entry_points::execute(deps.as_mut(), env.clone(), info.clone(), msg)
            .unwrap();        
    }

    let orders = query_all_orders(&deps, &env);
    assert!(orders.is_empty());

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
    let cows_per_pair = mantis_node::mantis::mantis::do_cows(orders);

    panic!("{:?}", cows_per_pair);

}

fn send_order(msg: ExecMsg, given: Coin, deps: &mut cosmwasm_std::OwnedDeps<cosmwasm_std::MemoryStorage, MockApi, MockQuerier>, env: &cosmwasm_std::Env) {
    let msg = cw_mantis_order::sv::ContractExecMsg::OrderContract(msg);
    let info = MessageInfo {
        funds: vec![given],
        sender: Addr::unchecked("sender"),
    };
    cw_mantis_order::entry_points::execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
}

fn query_all_orders(deps: &cosmwasm_std::OwnedDeps<cosmwasm_std::MemoryStorage, MockApi, MockQuerier>, env: &cosmwasm_std::Env) -> Vec<OrderItem> {
    let msg = QueryMsg::GetAllOrders {};
    let msg = cw_mantis_order::sv::ContractQueryMsg::OrderContract(msg);
    let orders: Binary =
        cw_mantis_order::entry_points::query(deps.as_ref(), env.clone(), msg).unwrap();
    let orders: Vec<OrderItem> = serde_json_wasm::from_slice(orders.as_slice()).unwrap();
    orders
}
