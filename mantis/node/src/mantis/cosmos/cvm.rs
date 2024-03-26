use super::client::Tip;

/// given key and latest block and sequence number, produces binary salt for cross chain block
/// isolating execution of one cross chain transaction from other
pub fn get_salt(signing_key: &cosmrs::crypto::secp256k1::SigningKey, tip:&Tip) -> Vec<u8> {
    let mut base = signing_key.public_key().to_bytes().to_vec();
    base.extend(tip.block.value().to_be_bytes().to_vec());
    base.extend(tip.account.sequence.to_be_bytes().to_vec());
    base
}