//! given whatever string, give me the signer struct

use cosmrs::{cosmwasm::MsgExecuteContract, crypto::secp256k1::SigningKey, tx};
use prost_types::Any;

pub fn from_mnemonic(phrase: &str, derivation_path: &str) -> Result<SigningKey, String> {
    let seed = bip32::Mnemonic::new(phrase, bip32::Language::English)
        .expect("mnemonic")
        .to_seed("");
    let xprv = bip32::XPrv::derive_from_path(seed, &derivation_path.parse().expect("parse"))
        .expect("derived");
    let signer_priv: SigningKey = xprv.into();
    Ok(signer_priv)
}
