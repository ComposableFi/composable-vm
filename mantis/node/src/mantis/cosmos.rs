use crate::prelude::*;

use cosmos_sdk_proto::cosmwasm::wasm::v1::msg_client::MsgClient;
use cosmos_sdk_proto::cosmwasm::wasm::v1::query_client::QueryClient;




use tonic::transport::Channel;

pub type WriteClient = MsgClient<Channel>;
pub type ReadClient = QueryClient<Channel>;

pub async fn create_wasm_query_client(rpc: &str) -> QueryClient<Channel> {
    let url = tonic::transport::Endpoint::from_str(rpc).expect("url");
    QueryClient::connect(url).await.expect("connected")
}

pub async fn create_wasm_write_client(rpc: &str) -> MsgClient<Channel> {
    let url = tonic::transport::Endpoint::from_str(rpc).expect("url");
    MsgClient::connect(url).await.expect("connected")
}
