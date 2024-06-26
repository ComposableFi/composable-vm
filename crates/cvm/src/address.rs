use crate::prelude::*;
use cosmwasm_std::{Api, Binary, CanonicalAddr, StdError};

/// A wrapper around any address on any chain.
/// Similar to `ibc_rs::Signer`(multi encoding), but not depend on ibc code bloat.
/// Unlike parity MultiLocation/Account32/Account20 which hard codes enum into code.
/// Better send canonical address to each chain for performance,
/// But it will also decode/reencode best effort.
/// Inner must be either base64 or hex encoded or contain only characters from these.
/// Added with helper per chain to get final address to use.
#[cfg_attr(
    feature = "json-schema", // all(feature = "json-schema", not(target_arch = "wasm32")),
    derive(schemars::JsonSchema)
)]
#[cfg_attr(
    feature = "scale",
    derive(
        parity_scale_codec::Encode,
        parity_scale_codec::Decode,
        scale_info::TypeInfo
    )
)]
#[derive(
    Clone,
    PartialEq,
    Eq,
    Hash,
    derive_more::Deref,
    derive_more::From,
    derive_more::Into,
    serde::Deserialize,
    serde::Serialize,
)]
#[into(owned, ref, ref_mut)]
#[repr(transparent)]
pub struct XcAddr(pub String);

impl From<XcAddr> for Vec<u8> {
    fn from(value: XcAddr) -> Self {
        value.0.into_bytes()
    }
}

impl TryFrom<Vec<u8>> for XcAddr {
    type Error = StdError;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        Ok(Self(String::from_utf8(value)?))
    }
}

impl XcAddr {
    /// idea that whatever user plugs into, it works, really for adoption
    /// sure for Ethereum he must plug exact binary address, but for others it's just a string
    #[cfg(feature = "cosmwasm")]
    pub fn encode_cosmwasm(&self, api: &dyn Api) -> Result<String, StdError> {
        let addr = self.parse()?;

        Ok(api.addr_humanize(&CanonicalAddr(addr))?.to_string())
    }
    #[cfg(feature = "cosmwasm")]
    pub fn from_cw_addr(addr: &cosmwasm_std::Addr) -> Self {
        Self(addr.to_string())
    }

    #[cfg(feature = "cosmwasm")]
    pub fn parse(&self) -> Result<Binary, StdError> {
        if let Ok((_, addr, _)) = bech32::decode(&self.0) {
            use bech32::FromBase32;
            if let Ok(addr) = Vec::from_base32(&addr) {
                return Ok(Binary(addr));
            }
        }
        Binary::from_base64(&self.0)
        // use bech32::{primitives::decode::CheckedHrpstring, Bech32};
        // let addr = if let Ok(addr) = CheckedHrpstring::new::<Bech32>(&self.0) {
        //     Binary(addr.byte_iter().collect())
        // } else if let Ok(addr) = Binary::from_base64(&self.0) {
        //     addr
        // } else {
        //     return Err(StdError::generic_err("Failed to ensure XcAddr encoding"));
        // };
        // Ok(addr)
    }
}

impl core::fmt::Display for XcAddr {
    fn fmt(&self, fmtr: &mut core::fmt::Formatter) -> core::fmt::Result {
        core::fmt::Display::fmt(&self.0, fmtr)
    }
}

impl core::fmt::Debug for XcAddr {
    fn fmt(&self, fmtr: &mut core::fmt::Formatter) -> core::fmt::Result {
        core::fmt::Debug::fmt(&self.0, fmtr)
    }
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "cosmwasm")]
    #[test]
    fn xcaddr() {
        let addr_a = "osmovalcons1qg7u70m2af8qpx9thg40y0eavkkryjz3rsxafg";
        let addr_b = "Aj3PP2rqTgCYq7oq8j89ZawySFE=";
        let addr_c = "cosmosvalcons1qg7u70m2af8qpx9thg40y0eavkkryjz35gfmyw";

        // this is valid base64 and this is very bad
        let addr_d = "023DCF3F6AEA4E0098ABBA2AF23F3D65AC324851";

        let xcaddr_a = super::XcAddr(addr_a.to_string());
        let xcaddr_b = super::XcAddr(addr_b.to_string());
        let xcaddr_c = super::XcAddr(addr_c.to_string());
        let _xcaddr_d = super::XcAddr(addr_d.to_string());
        assert_eq!(addr_b, xcaddr_a.parse().unwrap().to_base64());
        assert_eq!(addr_b, xcaddr_b.parse().unwrap().to_base64());
        assert_eq!(addr_b, xcaddr_c.parse().unwrap().to_base64());

        // next fails
        // assert_eq!(addr_b, xcaddr_d.parse().unwrap().to_base64());
    }
}
