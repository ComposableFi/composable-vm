//! actually simulates mantis
use cosmwasm_std::{testing::*, Addr};
use cw_mantis_order::sv::*;
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
fn solve_one() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info("sender", &[]);

    let mantis = InstantiateMsg {
        admin: Some(Addr::unchecked("sender".to_string())),
        cvm_address: Addr::unchecked("cows only".to_string()),
    };
}
