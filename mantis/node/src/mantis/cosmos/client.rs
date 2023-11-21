use crate::prelude::*;

use cosmos_sdk_proto::cosmos::{auth::v1beta1::QueryAccountRequest, base::tendermint};
use cosmrs::{
    auth::BaseAccount,
    rpc::Client,
    tendermint::{block::Height, chain},
    tx::{self, SignDoc},
};
use prost_types::Any;
use tonic::transport::Channel;

pub type CosmWasmWriteClient = cosmos_sdk_proto::cosmwasm::wasm::v1::msg_client::MsgClient<Channel>;
pub type CosmWasmReadClient =
    cosmos_sdk_proto::cosmwasm::wasm::v1::query_client::QueryClient<Channel>;

pub type CosmosQueryClient =
    cosmos_sdk_proto::cosmos::auth::v1beta1::query_client::QueryClient<Channel>;

pub async fn create_cosmos_query_client(rpc: &str) -> CosmosQueryClient {
    use cosmos_sdk_proto::cosmos::auth::v1beta1::query_client::*;
    use cosmos_sdk_proto::cosmos::auth::v1beta1::*;

    let url = tonic::transport::Endpoint::from_str(rpc).expect("url");
    QueryClient::connect(url).await.expect("connected")
}

pub async fn query_cosmos_account(
    rpc: &str,
    address: String,
) -> cosmos_sdk_proto::cosmos::auth::v1beta1::BaseAccount {
    use cosmos_sdk_proto::cosmos::auth::v1beta1::*;
    use cosmos_sdk_proto::traits::Message;
    let mut client = create_cosmos_query_client(rpc).await;
    let account = client
        .account(QueryAccountRequest { address })
        .await
        .expect("account")
        .into_inner();
    BaseAccount::decode(account.account.expect("some").value.as_slice()).expect("decode")
}

pub async fn create_wasm_query_client(rpc: &str) -> CosmWasmReadClient {
    let url = tonic::transport::Endpoint::from_str(rpc).expect("url");
    cosmos_sdk_proto::cosmwasm::wasm::v1::query_client::QueryClient::connect(url)
        .await
        .expect("connected")
}

pub async fn create_wasm_write_client(rpc: &str) -> CosmWasmWriteClient {
    let url = tonic::transport::Endpoint::from_str(rpc).expect("url");
    cosmos_sdk_proto::cosmwasm::wasm::v1::msg_client::MsgClient::connect(url)
        .await
        .expect("connected")
}

pub async fn get_latest_block_and_account(
    rpc: &str,
    address: String,
) -> (
    cosmrs::tendermint::block::Height,
    cosmos_sdk_proto::cosmos::auth::v1beta1::BaseAccount,
) {
    let rpc_client: cosmrs::rpc::HttpClient = cosmrs::rpc::HttpClient::new(rpc).unwrap();
    let status = rpc_client
        .status()
        .await
        .expect("status")
        .sync_info
        .latest_block_height;
    let account = query_cosmos_account(rpc, address).await;
    (status, account)
}

/// latest chain state
async fn get_latest_block(rpc: &str) -> cosmrs::tendermint::block::Height {
    use cosmrs::tendermint::block::Height;
    let rpc_client: cosmrs::rpc::HttpClient = cosmrs::rpc::HttpClient::new(rpc).unwrap();
    let status = rpc_client
        .status()
        .await
        .expect("status")
        .sync_info
        .latest_block_height;
    status
}

pub async fn sign_and_tx_tendermint(
    rpc: &str,
    sign_doc: SignDoc,
    signing_key: &cosmrs::crypto::secp256k1::SigningKey,
) -> cosmrs::rpc::endpoint::broadcast::tx_commit::Response {
    let rpc_client: cosmrs::rpc::HttpClient = cosmrs::rpc::HttpClient::new(rpc).unwrap();
    let tx_raw = sign_doc.sign(&signing_key).expect("signed");
    let result = tx_raw
        .broadcast_commit(&rpc_client)
        .await
        .expect("broadcasted");
    println!("result: {:?}", result);
    assert!(!result.check_tx.code.is_err(), "err");
    assert!(!result.tx_result.code.is_err(), "err");
    result
}

pub async fn tx_broadcast_single_signed_msg(
    msg: Any,
    block: Height,
    auth_info: tx::AuthInfo,
    account: cosmos_sdk_proto::cosmos::auth::v1beta1::BaseAccount,
    rpc: &str,
    signing_key: &cosmrs::crypto::secp256k1::SigningKey,
) {
    let tx_body = tx::Body::new(
        vec![msg],
        "",
        Height::try_from(block.value() + 100).unwrap(),
    );

    let sign_doc = SignDoc::new(
        &tx_body,
        &auth_info,
        &chain::Id::try_from("centauri-1").expect("id"),
        account.account_number,
    )
    .unwrap();

    sign_and_tx_tendermint(rpc, sign_doc, signing_key).await;
}


/// simulates tx and ensure fees are within limits
pub async fn simulate_and_set_fee(signing_key: &cosmrs::crypto::secp256k1::SigningKey, account: &BaseAccount) -> tx::AuthInfo {
    let auth_info = SignerInfo::single_direct(Some(signing_key.public_key()), account.sequence)
        .auth_info(Fee {
            amount: vec![cosmrs::Coin {
                amount: 10,
                denom: cosmrs::Denom::from_str("ppica").expect("denom"),
            }],
            gas_limit: 1_000_000,
            payer: None,
            granter: None,
        });
    auth_info
}