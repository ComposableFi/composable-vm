use crate::{Funds, UserOrigin};
use alloc::{string::String, vec::Vec};
#[cfg(feature = "cosmwasm")]
use cosmwasm_std::Binary;
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum XCVMAck {
    Ok,
    Fail,
}

impl XCVMAck {
    fn into_byte(self) -> u8 {
        match self {
            Self::Ok => 0,
            Self::Fail => 1,
        }
    }

    fn try_from_byte(value: u8) -> Result<Self, ()> {
        match value {
            0 => Ok(Self::Ok),
            1 => Ok(Self::Fail),
            _ => Err(()),
        }
    }
}

#[cfg(feature = "cosmwasm")]
impl From<XCVMAck> for Binary {
    fn from(value: XCVMAck) -> Self {
        Binary::from(Vec::from(value))
    }
}

impl From<XCVMAck> for Vec<u8> {
    fn from(value: XCVMAck) -> Self {
        [value.into_byte()].to_vec()
    }
}

impl From<XCVMAck> for String {
    fn from(ack: XCVMAck) -> Self {
        let digit = [b'0' + ack.into_byte()];
        // SAFETY: digit is always an ASCII digit
        Self::from(unsafe { core::str::from_utf8_unchecked(&digit[..]) })
    }
}

impl TryFrom<&[u8]> for XCVMAck {
    type Error = ();
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        match value {
            [byte] => Self::try_from_byte(*byte),
            _ => Err(()),
        }
    }
}

#[cfg_attr(
    feature = "json-schema", // all(feature = "json-schema", not(target_arch = "wasm32")),
    derive(schemars::JsonSchema)
)]
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Packet<Program> {
    /// The executor that was the origin of this packet.
    #[serde(with = "hex")]
    #[cfg_attr(
        feature = "json-schema", // all(feature = "json-schema", not(target_arch = "wasm32")),
        schemars(with = "String")
    )]
    pub executor: Vec<u8>,
    /// The user that originated the first CVM call.
    pub user_origin: UserOrigin,
    /// The salt associated with the program.
    #[serde(with = "hex")]
    #[cfg_attr(
        feature = "json-schema", // all(feature = "json-schema", not(target_arch = "wasm32")),
        schemars(with = "String")
    )]
    pub salt: Vec<u8>,
    /// The protobuf encoded program.
    pub program: Program,
    /// The assets that were attached to the program.
    pub assets: Funds<crate::shared::Displayed<u128>>,
}
