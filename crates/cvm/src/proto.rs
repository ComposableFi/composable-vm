include!(concat!(env!("OUT_DIR"), "/cvm.common.rs"));

impl From<super::proto::Uint128> for u128 {
    fn from(value: super::proto::Uint128) -> Self {
        ((value.high_bits as u128) << 64) | value.low_bits as u128
    }
}

impl From<u128> for super::proto::Uint128 {
    fn from(value: u128) -> Self {
        Self {
            high_bits: (value >> 64) as u64,
            low_bits: value as u64,
        }
    }
}

impl From<super::proto::Uint128> for crate::AssetId {
    fn from(value: super::proto::Uint128) -> Self {
        u128::from(value).into()
    }
}

impl From<crate::AssetId> for super::proto::Uint128 {
    fn from(value: crate::AssetId) -> Self {
        u128::from(value).into()
    }
}

#[test]
fn test_u128_uint128_conversion() {
    let value = 0xDEAD_0000_0000_0000_BEEF_0000_0000_0000_u128;
    let (high_bits, low_bits) = (0xDEAD_0000_0000_0000, 0xBEEF_0000_0000_0000);
    let msg = super::proto::Uint128 {
        high_bits,
        low_bits,
    };

    assert_eq!(msg, super::proto::Uint128::from(value));
    assert_eq!(value, u128::from(msg));
}
