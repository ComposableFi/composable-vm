use mantis_cw::DenomPair;

use super::client::Tip;

/// given key and latest block and sequence number, produces binary salt for cross chain block
/// isolating execution of one cross chain transaction from other
pub fn calculate_salt(signing_key: &cosmrs::crypto::secp256k1::SigningKey, tip: &Tip, pair: DenomPair) -> Vec<u8> {
    use sha2::{Sha256, Digest};
    let mut base = signing_key.public_key().to_bytes().to_vec();
    base.extend(tip.block.value().to_be_bytes().to_vec());
    base.extend(tip.account.sequence.to_be_bytes().to_vec());
    base.extend(pair.a.as_bytes().to_vec());
    base.extend(pair.b.as_bytes().to_vec());
    let mut hasher = Sha256::default();
    hasher.update(base);
    hasher.finalize().to_vec()
}
