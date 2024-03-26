use cosmrs::{tendermint::block::Height, tx::Msg, Gas};
use cw_mantis_order::{OrderItem, OrderSolution, OrderSubMsg};

use crate::{
    mantis::cosmos::{client::{simulate_and_set_fee, tx_broadcast_single_signed_msg}, cosmwasm::to_exec_signed_with_fund}, prelude::*, solver::{orderbook::OrderList, solution::Solution, types::OrderType}
};

use super::cosmos::client::{timeout, CosmWasmWriteClient, CosmosQueryClient, Tip};

pub fn randomize_order(
    coins_pair: String,
    tip: Height,
) -> (cw_mantis_order::ExecMsg, cosmrs::Coin) {
    let coins = fun_name(coins_pair);

    let coins = if rand::random::<bool>() {
        (coins[0].clone(), coins[1].clone())
    } else {
        (coins[1].clone(), coins[0].clone())
    };
    let coin_0_random = randomize_coin(coins.0.amount.u128());
    let coin_1_random = randomize_coin(coins.1.amount.u128());

    let msg = cw_mantis_order::ExecMsg::Order {
        msg: OrderSubMsg {
            wants: cosmwasm_std::Coin {
                amount: coin_0_random.into(),
                denom: coins.0.denom.clone(),
            },
            transfer: None,
            timeout: timeout(tip, 100),
            min_fill: None,
        },
    };
    let fund = cosmrs::Coin {
        amount: coin_1_random.into(),
        denom: cosmrs::Denom::from_str(&coins.1.denom).expect("denom"),
    };
    (msg, fund)
}


pub fn randomize_coin(coin_0_amount: u128) -> u128 {
    let delta_0 = 1.max(coin_0_amount / 10);
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
    write_client: &mut CosmWasmWriteClient,
    cosmos_query_client: &mut CosmosQueryClient,
    order_contract: String,
    coins_pair: String,
    signing_key: &cosmrs::crypto::secp256k1::SigningKey,
    rpc: &str,
    tip: &Tip,
    gas: Gas,
) {
    log::info!("========================= simulate_order =========================");
    let (msg, fund) = randomize_order(coins_pair, tip.block);

    println!("msg: {:?}", msg);

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