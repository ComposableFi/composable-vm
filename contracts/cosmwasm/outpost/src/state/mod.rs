pub mod assets;
pub mod exchange;
pub mod ics27;
pub mod interpreter;
pub mod network;
pub mod tracking;
use crate::{error::ContractError, prelude::*};

use cosmwasm_std::{StdResult, Storage};
use cvm_route::transport::OtherNetworkItem;
use cvm_runtime::outpost::{GetConfigResponse, NetworkItem};
use cw_storage_plus::Item;

use cvm_runtime::NetworkId;

const CONFIG: Item<HereItem> = Item::new("this");

pub(crate) fn load(storage: &dyn Storage) -> StdResult<HereItem> {
    CONFIG.load(storage)
}

pub(crate) fn save(storage: &mut dyn Storage, value: &HereItem) -> StdResult<()> {
    CONFIG.save(storage, value)
}

pub(crate) fn get_config(deps: cosmwasm_std::Deps<'_>) -> Result<GetConfigResponse, ContractError> {
    let exchanges = exchange::get_all_exchanges(deps)?;
    panic!()
}
