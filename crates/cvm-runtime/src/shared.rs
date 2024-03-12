use crate::{prelude::*, AssetId};
use cosmwasm_std::{from_json, to_json_binary, Api, Binary, CanonicalAddr, StdError, StdResult};
use serde::{de::DeserializeOwned, Serialize};

pub use cvm::shared::*;
pub type Salt = Vec<u8>;
/// absolute amounts
pub type CvmFunds = Vec<(AssetId, Displayed<u128>)>;
/// like `CvmFunds`, but allow relative(percentages) amounts. Similar to assets filters in XCM
pub type CvmBalanceFilter = crate::asset::Amount;
pub type CvmFundsFilter = crate::Funds<CvmBalanceFilter>;
pub type CvmInstruction = crate::Instruction<Vec<u8>, XcAddr, CvmFundsFilter>;
pub type CvmPacket = crate::Packet<CvmProgram>;
pub type CvmProgram = crate::Program<Vec<CvmInstruction>>;

impl CvmProgram {
    fn new(instructions: Vec<CvmInstruction>) -> Self {
        Self {
            tag: vec![0],
            instructions,
        }     
}
}


impl CvmInstruction {
    pub fn transfer_absolute_to_account(to: &str, asset_id: u128, amount: u128) -> Self {
        Self::Transfer {
            to: crate::Destination::Account(XcAddr(to.to_owned())),
            assets: CvmFundsFilter::one(asset_id.into(), crate::Amount::new(amount, 0)),
        }
    }
}

pub fn to_json_base64<T: Serialize>(x: &T) -> StdResult<String> {
    Ok(to_json_binary(x)?.to_base64())
}

pub fn decode_base64<S: AsRef<str>, T: DeserializeOwned>(encoded: S) -> StdResult<T> {
    from_json::<T>(&Binary::from_base64(encoded.as_ref())?)
}
