use cosmrs::{tendermint::block::Height, tx::Msg, Gas};
use cw_mantis_order::OrderSubMsg;

use crate::{
    mantis::cosmos::{
        client::{simulate_and_set_fee, tx_broadcast_single_signed_msg},
        cosmwasm::to_exec_signed_with_fund,
    },
    prelude::*,
};

use super::cosmos::{
    client::{timeout, CosmWasmWriteClient, CosmosChainInfo, CosmosQueryClient, Tip},
    cosmwasm::parse_coin_pair,
};

pub fn randomize_order(
    pair: &String,
    tip: Height,
    random_parts: u8,
) -> (cw_mantis_order::ExecMsg, cosmrs::Coin) {
    let pair = parse_coin_pair(pair);

    let pair = if rand::random::<bool>() {
        (pair.0.clone(), pair.1.clone())
    } else {
        (pair.1.clone(), pair.0.clone())
    };
    let coin_0_random = randomize_coin(pair.0.amount.u128(), random_parts);
    let coin_1_random = randomize_coin(pair.1.amount.u128(), random_parts);

    let msg = cw_mantis_order::ExecMsg::Order {
        msg: OrderSubMsg {
            wants: cosmwasm_std::Coin {
                amount: coin_0_random.into(),
                denom: pair.0.denom.clone(),
            },
            convert: None,
            timeout: timeout(tip, 100),
            min_fill: None,
            virtual_given: None,
        },
    };
    let fund = cosmrs::Coin {
        amount: coin_1_random.into(),
        denom: cosmrs::Denom::from_str(&pair.1.denom).expect("denom"),
    };
    (msg, fund)
}

pub fn randomize_coin(coin_0_amount: u128, random_parts: u8) -> u128 {
    let delta_0 = 1.max(coin_0_amount / random_parts as u128);

    let coin_0_random = rand_distr::Uniform::new(coin_0_amount - delta_0, coin_0_amount + delta_0);
    let coin_0_random: u128 = coin_0_random.sample(&mut rand::thread_rng());
    coin_0_random
}

/// `assets` - is comma separate list. each entry is amount u64 glued with alphanumeric denomination
/// that is splitted into array of CosmWasm coins.
/// one coin is chosen as given,
/// from remaining 2 other coins one is chosen as wanted
/// amount of count is randomized around values
///
/// `write_client`
/// `order_contract` - orders are formed for give and want, and send to orders contract.
/// timeout is also randomized starting from 10 to 100 blocks
///
/// Also calls `timeout` so old orders are cleaned.
pub async fn simulate_order(
    _write_client: &mut CosmWasmWriteClient,
    _cosmos_query_client: &mut CosmosQueryClient,
    order_contract: String,
    coins_pair: &String,
    signing_key: &cosmrs::crypto::secp256k1::SigningKey,
    rpc: &CosmosChainInfo,
    tip: &Tip,
    gas: Gas,
    random_parts: u8,
) {
    log::info!("========================= simulate_order =========================");
    let (msg, fund) = randomize_order(&coins_pair, tip.block, random_parts);

    log::info!("msg: {:?}", msg);

    let auth_info = simulate_and_set_fee(signing_key, &tip.account, gas).await;

    let msg = to_exec_signed_with_fund(signing_key, order_contract, msg, fund);

    let result = tx_broadcast_single_signed_msg(
        msg.to_any().expect("proto"),
        auth_info,
        rpc,
        signing_key,
        &tip,
    )
    .await;

    println!("simulated tx {:?}", result.height)
}
