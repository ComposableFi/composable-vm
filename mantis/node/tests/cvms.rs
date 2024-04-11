use cosmwasm_std::{Addr, Empty};
use cw_cvm_outpost::msg::HereItem;
use cw_multi_test::{App, Contract, ContractWrapper, Executor};
// use cw_orch::prelude::*;
// use cw_orch::interface;

#[test]
fn cvm_devnet_case() {
    let mut centauri = App::default();
    let mut _osmosis = App::default();
    let cw_mantis_order_wasm = (ContractWrapper::new(
        cw_mantis_order::entry_points::execute,
        cw_mantis_order::entry_points::instantiate,
        cw_mantis_order::entry_points::query,
    ));

    let cw_cvm_outpost_wasm = (ContractWrapper::new(
        cw_cvm_outpost::contract::execute::execute,
        cw_cvm_outpost::contract::instantiate,
        cw_cvm_outpost::contract::query::query,
    ));

    let cw_cvm_executor_wasm = (ContractWrapper::new(
        cw_cvm_executor::contract::execute,
        cw_cvm_executor::contract::instantiate,
        cw_cvm_executor::contract::query,
    ));

    let cw_mantis_order_code_id = centauri.store_code(Box::new(cw_mantis_order_wasm));
    let cw_cvm_outpost_code_id = centauri.store_code(Box::new(cw_cvm_outpost_wasm));
    let cw_cvm_executor_code_id = centauri.store_code(Box::new(cw_cvm_executor_wasm));

    let admin = Addr::unchecked("juno1g2rahf5846rxzp3fwlswy08fz8ccuwk03k57y");
    let cw_cvm_outpost_instantiate = cw_cvm_outpost::msg::InstantiateMsg(HereItem {
        network_id: 3.into(),
        admin: admin.clone(),
    });
    let cw_cvm_outpost_contract = centauri
        .instantiate_contract(
            cw_cvm_outpost_code_id,
            admin.clone(),
            &cw_cvm_outpost_instantiate,
            &[],
            "composable_cvm_outpost",
            None,
        )
        .unwrap();

    let cw_mantis_order_instantiate = cw_mantis_order::sv::InstantiateMsg {
        admin: Some(admin.clone()),
        cvm_address: cw_cvm_outpost_contract.clone(),
    };

    
    let cw_mantis_contract = centauri
    .instantiate_contract(
        cw_mantis_order_code_id,
        admin,
        &cw_mantis_order_instantiate,
        &[],
        "composable_mantis_order",
        None,
    )
    .unwrap();

    let sender = Addr::unchecked("juno16g2rahf5846rxzp3fwlswy08fz8ccuwk03k57y");
    
    let solution = mantis_node::mantis::blackbox::solve(active_orders, signing_key, tip, cvm_glt, router);
}