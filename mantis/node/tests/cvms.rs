use bounded_collections::Get;
use cosmrs::tendermint::block::Height;
use cosmwasm_std::{Addr, Coin, Coins, Empty, Querier, QuerierWrapper};
use cvm_route::{
    asset::{self, AssetItem, AssetReference, NetworkAssetItem},
    exchange::ExchangeItem,
    transport::{NetworkToNetworkItem, OtherNetworkItem},
    venue::AssetsVenueItem,
};
use cw_cvm_outpost::msg::{CvmGlt, GetAssetResponse, HereItem, NetworkItem, OutpostId};
use cw_mantis_order::{OrderItem, OrderSubMsg, SolutionSubMsg};
use cw_multi_test::{App, Bank, BankKeeper, Contract, ContractWrapper, Executor, StargateQuery};
use mantis_node::mantis::cosmos::{client::Tip, signer::from_mnemonic};
use serde::de;

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

    let sender = Addr::unchecked("juno1g2rahf5846rxzp3fwlswy08fz8ccuwk03k57y");
    let cw_cvm_outpost_instantiate = cw_cvm_outpost::msg::InstantiateMsg(HereItem {
        network_id: 1.into(),
        admin: sender.clone(),
    });
    let cw_cvm_outpost_contract = centauri
        .instantiate_contract(
            cw_cvm_outpost_code_id,
            sender.clone(),
            &cw_cvm_outpost_instantiate,
            &[],
            "composable_cvm_outpost",
            None,
        )
        .unwrap();

    let cw_mantis_order_instantiate = cw_mantis_order::sv::InstantiateMsg {
        admin: Some(sender.clone()),
        cvm_address: cw_cvm_outpost_contract.clone(),
    };

    let cw_mantis_contract = centauri
        .instantiate_contract(
            cw_mantis_order_code_id,
            sender.clone(),
            &cw_mantis_order_instantiate,
            &[],
            "composable_mantis_order",
            None,
        )
        .unwrap();

    let ACoin = |x: u128| Coin {
        denom: "a".to_string(),
        amount: x.into(),
    };

    let BCoin = |x: u128| Coin {
        denom: "ibc/b".to_string(),
        amount: x.into(),
    };

    let bank = BankKeeper::new();
    bank.init_balance(
        centauri.storage_mut(),
        &sender,
        vec![ACoin(10u128.pow(10)), BCoin(10u128.pow(10))],
    )
    .unwrap();

    let a_to_b_msg = OrderSubMsg {
        wants: BCoin(100),
        timeout: centauri.block_info().height + 100,
        convert: None,
        min_fill: None,
        virtual_given: None,
    };
    let a_to_b = OrderItem {
        owner: sender.clone(),
        msg: a_to_b_msg.clone(),
        given: ACoin(100),
        order_id: 0u128.into(),
    };

    let b_to_a_msg = OrderSubMsg {
        wants: ACoin(1000),
        timeout: centauri.block_info().height + 100,
        convert: None,
        min_fill: None,
        virtual_given: None,
    };
    let b_to_a = OrderItem {
        owner: sender.clone(),
        msg: b_to_a_msg.clone(),
        given: BCoin(1000),
        order_id: 1u128.into(),
    };

    centauri
        .execute_contract(
            sender.clone(),
            cw_mantis_contract.clone(),
            &cw_mantis_order::sv::ExecMsg::Order {
                msg: a_to_b_msg.clone(),
            },
            &[ACoin(100)],
        )
        .unwrap();

    centauri
        .execute_contract(
            sender.clone(),
            cw_mantis_contract.clone(),
            &cw_mantis_order::sv::ExecMsg::Order {
                msg: b_to_a_msg.clone(),
            },
            &[BCoin(1000)],
        )
        .unwrap();

    let active_orders = vec![a_to_b, b_to_a];
    let alice = from_mnemonic(
        "document prefer nurse marriage flavor cheese west when knee drink sorry minimum thunder tilt cherry behave cute stove elder couch badge gown coral expire", 
    "m/44'/118'/0'/0/0",).unwrap();
    let tip = Tip {
        block: Height::default(),
        account: cosmos_sdk_proto::cosmos::auth::v1beta1::BaseAccount {
            address: alice.public_key().to_string(),
            pub_key: Some(alice.public_key().to_any().unwrap()),
            account_number: 1,
            sequence: 1,
        },
    };

    let router = "shortest_path";
    let cvm_glt = CvmGlt {
        network_to_networks: vec![
            NetworkToNetworkItem::new(1.into(), 2.into(), OtherNetworkItem::new()),
            NetworkToNetworkItem::new(2.into(), 1.into(), OtherNetworkItem::new()),
        ],
        assets: vec![
            AssetItem::new(
                11.into(),
                1.into(),
                AssetReference::Native {
                    denom: "a".to_string(),
                },
            ),
            AssetItem::new(
                12.into(),
                1.into(),
                AssetReference::Native {
                    denom: "ibc/b".to_string(),
                },
            ),
            AssetItem::new(
                21.into(),
                2.into(),
                AssetReference::Native {
                    denom: "b".to_string(),
                },
            ),
            AssetItem::new(
                22.into(),
                2.into(),
                AssetReference::Native {
                    denom: "ibc/a".to_string(),
                },
            ),
        ],
        exchanges: vec![ExchangeItem::new(
            1.into(),
            2.into(),
            cvm_route::exchange::ExchangeType::OsmosisPoolManagerModuleV1Beta1 {
                pool_id: 1,
                token_a: "b".to_string(),
                token_b: "ibc/a".to_string(),
            },
        )],
        networks: vec![
            NetworkItem {
                network_id: 1.into(),
                outpost: Some(OutpostId::CosmWasm {
                    contract: cw_cvm_outpost_contract.clone(),
                    executor_code_id: cw_cvm_executor_code_id,
                    admin: sender.clone(),
                }),
                accounts: None,
                ibc: None,
            },
            NetworkItem {
                network_id: 2.into(),
                outpost: Some(OutpostId::CosmWasm {
                    contract: cw_cvm_outpost_contract.clone(),
                    executor_code_id: cw_cvm_executor_code_id,
                    admin: sender.clone(),
                }),
                accounts: None,
                ibc: None,
            },
        ],
        network_assets: vec![
            NetworkAssetItem::new(2.into(), 11.into(), 22.into()),
            NetworkAssetItem::new(2.into(), 12.into(), 21.into()),
            NetworkAssetItem::new(1.into(), 21.into(), 12.into()),
            NetworkAssetItem::new(1.into(), 22.into(), 11.into()),
        ],
        asset_venue_items: vec![
            AssetsVenueItem::new(
                cvm_route::venue::VenueId::Exchange(1.into()),
                21.into(),
                22.into(),
            ),
            AssetsVenueItem::new(
                cvm_route::venue::VenueId::Exchange(1.into()),
                22.into(),
                21.into(),
            ),
        ],
    };

    let mut config_messages = vec![];

    for network in cvm_glt.networks.clone().into_iter() {
        let config_message = cw_cvm_outpost::msg::ConfigSubMsg::ForceNetwork(network);
        config_messages.push(config_message);
    }

    for asset in cvm_glt.assets.clone().into_iter() {
        let config_message = cw_cvm_outpost::msg::ConfigSubMsg::ForceAsset(asset);
        config_messages.push(config_message);
    }

    for network_to_network in cvm_glt.network_to_networks.clone().into_iter() {
        let config_message =
            cw_cvm_outpost::msg::ConfigSubMsg::ForceNetworkToNetwork(network_to_network);
        config_messages.push(config_message);
    }

    for exchange in cvm_glt.exchanges.clone().into_iter() {
        let config_message = cw_cvm_outpost::msg::ConfigSubMsg::ForceExchange(exchange);
        config_messages.push(config_message);
    }

    for asset_venue in cvm_glt.asset_venue_items.clone().into_iter() {
        let config_message = cw_cvm_outpost::msg::ConfigSubMsg::ForceAssetsVenue(asset_venue);
        config_messages.push(config_message);
    }

    for network_asset in cvm_glt.network_assets.clone().into_iter() {
        let config_message =
            cw_cvm_outpost::msg::ConfigSubMsg::ForceAssetToNetworkMap(network_asset);
        config_messages.push(config_message);
    }

    let force_config = cw_cvm_outpost::msg::ExecuteMsg::Config(
        cw_cvm_outpost::msg::ConfigSubMsg::Force(config_messages),
    );

    centauri
        .execute_contract(
            sender.clone(),
            cw_cvm_outpost_contract.clone(),
            &force_config,
            &[],
        )
        .unwrap();

    let solution = mantis_node::mantis::blackbox::solve::<True>(
        active_orders,
        &alice,
        &tip,
        cvm_glt.into(),
        router,
    )
    .await;

    centauri
        .execute_contract(
            sender.clone(),
            cw_mantis_contract.clone(),
            &solution[0],
            &[],
        )
        .unwrap();

    centauri.update_block(|x| {
        x.height += 2;
    });

    let query = cvm_runtime::outpost::QueryMsg::GetAllAssetIds {};
    let cvm_glt: Vec<AssetItem> = centauri
        .wrap()
        .query_wasm_smart(cw_cvm_outpost_contract, &query)
        .unwrap();
    //panic!("{:?}", cvm_glt);
    centauri
        .execute_contract(
            sender.clone(),
            cw_mantis_contract.clone(),
            &solution[0],
            &[],
        )
        .expect("https://github.com/CosmWasm/cw-multi-test/blob/main/src/wasm.rs#L722");
}

enum True {}

impl Get<bool> for True {
    fn get() -> bool {
        true
    }
}
