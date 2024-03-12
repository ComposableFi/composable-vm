use core::panic;

use crate::{prelude::*, AssetId};
use cosmwasm_std::{from_json, to_json_binary, Api, Binary, CanonicalAddr, StdError, StdResult};
use cvm::NetworkId;
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


pub type CvmSpawnRef<'a> = (&'a CvmProgram,&'a CvmFundsFilter);   

impl CvmProgram {
    pub fn new(instructions: Vec<CvmInstruction>) -> Self {
        Self {
            tag: vec![0],
            instructions,
        }
    }

    pub fn will_spawn(&self) -> bool {
        self.instructions
            .iter()
            .any(|i| matches!(i, CvmInstruction::Spawn { .. }))
    }

    pub fn last_spawns(&self) -> Vec<CvmSpawnRef> {
        self.instructions
            .iter()
            .filter_map(|i| {
                if let CvmInstruction::Spawn { program, assets, .. } = i {
                    if program.will_spawn() {
                        Some(program.last_spawns())
                    } else {
                        Some(vec![(program, assets)])
                    }
                } else {
                    None
                }
            })
            .flatten()
            .collect()
    }
}

impl CvmInstruction {
    pub fn transfer_absolute_to_account(to: &str, asset_id: u128, amount: u128) -> Self {
        Self::Transfer {
            to: crate::Destination::Account(XcAddr(to.to_owned())),
            assets: CvmFundsFilter::one(asset_id.into(), crate::Amount::new(amount, 0)),
        }
    }

    pub fn spawn(network_id: NetworkId, program: CvmProgram ) -> Self {
        Self::Spawn {
            network_id,
            salt: vec![],
            assets: vec![],
            program,
        }
    }
}

pub fn to_json_base64<T: Serialize>(x: &T) -> StdResult<String> {
    Ok(to_json_binary(x)?.to_base64())
}

pub fn decode_base64<S: AsRef<str>, T: DeserializeOwned>(encoded: S) -> StdResult<T> {
    from_json::<T>(&Binary::from_base64(encoded.as_ref())?)
}


#[cfg(test)]
mod tests {
    use super::{CvmInstruction, CvmProgram};


    #[test]
    pub fn spawn() {
        let spawn = CvmInstruction::Spawn { network_id: (), salt: (), assets: (), program: () }
        let program = CvmProgram::new(vec![

        ]);
    }   
}