use bounded_collections::Get;
use cosmrs::tendermint::block::Height;
use cosmwasm_std::{Addr, Coin, Coins, Empty};
use cw_cvm_outpost::msg::{CvmGlt, HereItem};
use cw_mantis_order::{OrderItem, OrderSubMsg};
use cw_multi_test::{App, Contract, ContractWrapper, Executor};
use mantis_node::mantis::cosmos::{client::Tip, signer::from_mnemonic};
use serde::de;
// use cw_orch::prelude::*;
// use cw_orch::interface;

#[tokio::test]
async fn cvm_devnet_case() {
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
    
    let ACoin = |x: u128| Coin {
        denom: "a".to_string(),
        amount: x.into(),
    };

    let BCoin = |x : u128| Coin {
        denom: "b".to_string(),
        amount: x.into(),
    };

    let a_to_b = OrderItem {
        owner: sender.clone(),
        msg: OrderSubMsg {
            wants: ACoin(100),
            timeout: centauri.block_info().height+100,
            convert: None,
            min_fill: None,
            virtual_given: None,
            
        },
        given: BCoin(100),
        order_id: 1u128.into(),
    };

    let b_to_a = OrderItem {
        owner: sender.clone(),
        msg: OrderSubMsg {
            wants: BCoin(1000),
            timeout: centauri.block_info().height+100,
            convert: None,
            min_fill: None,
            virtual_given: None,

        },
        given: ACoin(1000),
        order_id: 2u128.into(),
    };
    let active_orders = vec![a_to_b, b_to_a];
    let alice = from_mnemonic(
        "document prefer nurse marriage flavor cheese west when knee drink sorry minimum thunder tilt cherry behave cute stove elder couch badge gown coral expire"
    , 
    "m/44'/118'/0'/0/0",).unwrap();
    let tip = Tip {
        block: Height::default(),
        account: cosmos_sdk_proto::cosmos::auth::v1beta1::BaseAccount{
            address: alice.public_key().to_string(),
            pub_key: Some(alice.public_key().to_any().unwrap()),
            account_number: 1,
            sequence: 1,
        },
    };
    let router = "shortest_path";
    let cvm_glt = Some(CvmGlt {
        network_to_networks: todo!(),
        assets: todo!(),
        exchanges: todo!(),
        networks: todo!(),
        network_assets: todo!(),
        asset_venue_items: todo!(),
    });
    let solution = mantis_node::mantis::blackbox::solve::<True>(active_orders, &alice, &tip, cvm_glt, router);
}

enum True{}

impl Get<bool> for True {
    fn get() -> bool {
        true
    }
}
