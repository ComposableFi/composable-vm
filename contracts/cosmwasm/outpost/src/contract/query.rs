use crate::{assets, error::Result, msg};

use cosmwasm_std::{to_json_binary, Binary, Deps, Env};
use cvm_runtime::outpost::GetConfigResponse;

use super::ibc::ics20::get_this_route;

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn query(deps: Deps, _env: Env, msg: msg::QueryMsg) -> Result<Binary> {
    use msg::QueryMsg::*;
    match msg {
        GetAssetById { asset_id } => assets::get_asset_by_id(deps, asset_id)
            .and_then(|asset| Ok(to_json_binary(&msg::GetAssetResponse { asset })?)),
        GetLocalAssetByReference { reference } => {
            assets::get_local_asset_by_reference(deps, reference)
                .and_then(|asset| Ok(to_json_binary(&msg::GetAssetResponse { asset })?))
        }
        GetIbcIcs20Route {
            to_network,
            for_asset,
        } => get_this_route(deps.storage, to_network, for_asset)
            .and_then(|route| Ok(to_json_binary(&msg::GetIbcIcs20RouteResponse { route })?)),
        GetExchangeById { exchange_id } => crate::state::exchange::get_by_id(deps, exchange_id)
            .and_then(|exchange| Ok(to_json_binary(&msg::GetExchangeResponse { exchange })?)),
        GetConfig {} => {
            crate::state::get_config(deps).and_then(|config| Ok(to_json_binary(&config)?))
        } // GetRoute { program } => router::get_route(deps, program),
    }
}
