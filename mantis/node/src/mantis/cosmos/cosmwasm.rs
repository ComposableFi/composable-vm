use std::str::FromStr;

use cosmos_sdk_proto::cosmwasm::{self, wasm::v1::QuerySmartContractStateRequest};
use cosmrs::{cosmwasm::MsgExecuteContract, AccountId};

pub fn to_exec_signed_with_fund<T: serde::ser::Serialize>(
    signing_key: &cosmrs::crypto::secp256k1::SigningKey,
    order_contract: String,
    msg: T,
    fund: cosmrs::Coin,
) -> MsgExecuteContract {
    let funds = vec![fund];
    to_exec_signed_with_funds(signing_key, order_contract, msg, funds)
}

pub fn to_exec_signed<T: serde::ser::Serialize>(
    signing_key: &cosmrs::crypto::secp256k1::SigningKey,
    order_contract: String,
    msg: T,
) -> MsgExecuteContract {
    to_exec_signed_with_funds(signing_key, order_contract, msg, vec![])
}

pub fn to_exec_signed_with_funds<T: serde::ser::Serialize>(
    signing_key: &cosmrs::crypto::secp256k1::SigningKey,
    order_contract: String,
    msg: T,
    funds: Vec<cosmrs::Coin>,
) -> MsgExecuteContract {
    let msg = MsgExecuteContract {
        sender: signing_key
            .public_key()
            .account_id("centauri")
            .expect("account"),
        contract: AccountId::from_str(&order_contract).expect("contract"),
        msg: serde_json_wasm::to_vec(&msg).expect("json"),
        funds,
    };
    msg
}

pub async fn smart_query<T: serde::ser::Serialize, O: serde::de::DeserializeOwned>(
    order_contract: &String,
    query: T,
    read: &mut cosmwasm::wasm::v1::query_client::QueryClient<tonic::transport::Channel>,
) -> O {
    let orders_request = QuerySmartContractStateRequest {
        address: order_contract.clone(),
        query_data: serde_json_wasm::to_vec(&query).expect("json"),
    };
    let result = read
        .smart_contract_state(orders_request)
        .await
        .expect("result obtained")
        .into_inner()
        .data;
    serde_json_wasm::from_slice(&result).expect("result parsed")
}
