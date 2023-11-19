use crate::prelude::*;

use cosmos_sdk_proto::cosmos::auth::v1beta1::QueryAccountRequest;
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

pub async fn query_cosmos_account(rpc: &str, address : String) -> cosmos_sdk_proto::cosmos::auth::v1beta1::BaseAccount {
    use cosmos_sdk_proto::traits::Message;
    use cosmos_sdk_proto::cosmos::auth::v1beta1::*;
    let mut client = create_cosmos_query_client(rpc).await;
    let account = client.account(QueryAccountRequest {
        address,
    }).await.expect("account").into_inner();
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
