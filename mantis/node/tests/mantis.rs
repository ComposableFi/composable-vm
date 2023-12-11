//! actually simulates mantis
use cosmwasm_std::{testing::*, Addr, Coin, MessageInfo};
use cvm_runtime::executor::ExecuteMsg;
use cw_mantis_order::{sv::*, OrderSubMsg};
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

    let msg = ExecMsg::Order {
        msg: OrderSubMsg {
            wants: Coin {
                denom: "a".to_string(),
                amount: 20000u128.into(),
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
    // 2 200000
    // same by more
}
