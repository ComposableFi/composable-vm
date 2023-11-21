use std::str::FromStr;

use cosmrs::{AccountId, cosmwasm::MsgExecuteContract};


pub fn to_exec_signed_with_fund<T:serde::ser::Serialize>(signing_key: &cosmrs::crypto::secp256k1::SigningKey, order_contract: String, msg: T, fund: cosmrs::Coin) -> MsgExecuteContract {
    let msg = MsgExecuteContract {
        sender: signing_key
            .public_key()
            .account_id("centauri")
            .expect("account"),
        contract: AccountId::from_str(&order_contract).expect("contract"),
        msg: serde_json_wasm::to_vec(&msg).expect("json"),
        funds: vec![fund],
    };
    msg
}
