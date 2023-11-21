//! given whatever string, give me the signer struct

use cosmrs::{crypto::secp256k1::SigningKey, cosmwasm::MsgExecuteContract, tx};

pub fn from_mnemonic(phrase: &str, derivation_path: &str) -> Result<SigningKey, String> {
    let seed = bip32::Mnemonic::new(phrase, bip32::Language::English)
        .expect("mnemonic")
        .to_seed("");
    let xprv = bip32::XPrv::derive_from_path(seed, &derivation_path.parse().expect("parse"))
        .expect("derived");
    let signer_priv: SigningKey = xprv.into();
    Ok(signer_priv)
}



pub async fn tx_broadcast_single_signed_msg(msg: MsgExecuteContract, block: Height, auth_info: tx::AuthInfo, account: BaseAccount, rpc: &str, signing_key: &cosmrs::crypto::secp256k1::SigningKey) {
    let msg = msg.to_any().expect("proto");

    let tx_body = tx::Body::new(
        vec![msg],
        "mantis-solver",
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