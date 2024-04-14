pub mod assets;
pub mod exchange;
pub mod executors;
pub mod ics27;
pub mod network;
pub mod tracking;
use crate::{error::ContractError, prelude::*};

use cosmwasm_std::{StdResult, Storage};

use cvm_route::asset;
use cvm_runtime::outpost::GetConfigResponse;
use cw_storage_plus::Item;

const CONFIG: Item<HereItem> = Item::new("this");

pub(crate) fn load(storage: &dyn Storage) -> StdResult<HereItem> {
    CONFIG.load(storage)
}

pub(crate) fn save(storage: &mut dyn Storage, value: &HereItem) -> StdResult<()> {
    CONFIG.save(storage, value)
}

pub(crate) fn get_config(deps: cosmwasm_std::Deps<'_>) -> Result<GetConfigResponse, ContractError> {
    use crate::state::*;
    let network_to_networks = network::get_all_network_to_network(deps)?;
    deps.api.debug("got networks to networks");
    let exchanges = exchange::get_all_exchanges(deps)?;
    deps.api.debug("got exchanges");
    let assets = assets::get_all_assets(deps)?;
    deps.api.debug("got assets");
    let networks = network::get_all_networks(deps)?;
    deps.api.debug("got networks");
    let network_assets = assets::get_all_network_assets(deps)?;
    deps.api.debug("got network assets");
    let asset_venue_items = exchange::get_all_exchange_venues(deps)?;
    deps.api.debug("got asset venue items");
    let get_config_response = GetConfigResponse {
        network_to_networks,
        assets,
        exchanges,
        networks,
        network_assets,
        asset_venue_items,
    };
    Ok(get_config_response)
}
