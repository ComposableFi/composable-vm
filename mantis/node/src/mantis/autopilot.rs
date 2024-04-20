use cosmrs::{tx::Msg, Gas};

use crate::{
    mantis::cosmos::{
        client::{simulate_and_set_fee, tx_broadcast_single_signed_msg},
        cosmwasm::to_exec_signed,
    },
    prelude::*,
};

use super::cosmos::client::{CosmWasmWriteClient, CosmosChainInfo, CosmosQueryClient, Tip};

pub async fn cleanup(
    _write_client: &mut CosmWasmWriteClient,
    _cosmos_query_client: &mut CosmosQueryClient,
    order_contract: String,
    signing_key: &cosmrs::crypto::secp256k1::SigningKey,
    rpc: &CosmosChainInfo,
    tip: &Tip,
    gas: Gas,
) {
    log::info!("========================= cleanup =========================");
    let auth_info = simulate_and_set_fee(signing_key, &tip.account, gas).await;
    let msg = cw_mantis_order::ExecMsg::Timeout {
        orders: vec![],
        solutions: vec![],
    };
    let msg = to_exec_signed(signing_key, order_contract, msg);
    let result = tx_broadcast_single_signed_msg(
        msg.to_any().expect("proto"),
        auth_info,
        rpc,
        signing_key,
        tip,
    )
    .await;
    match &result.tx_result.code {
        cosmrs::tendermint::abci::Code::Err(err) => {
            log::error!("clean result: {:?}", result);
        }
        cosmrs::tendermint::abci::Code::Ok => {
            log::trace!("ok: {:?}", result);
        }
    }
}

/// move protocol forwards, cranks auctions ending and also cleans up old orders
async fn _move() {}
