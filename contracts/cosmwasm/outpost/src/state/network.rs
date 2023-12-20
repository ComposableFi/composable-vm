use crate::{
    batch::BatchResponse, events::make_event, prelude::*, state::ics27::IBC_CHANNEL_NETWORK,
};
use cosmwasm_std::{DepsMut, Storage, Deps, StdResult, Order};
use cvm_route::transport::*;
use cvm_runtime::{outpost::NetworkItem, NetworkId};

use crate::state::{self};

use crate::error::{ContractError, Result};

pub fn load_this(storage: &dyn Storage) -> Result<NetworkItem> {
    state::load(storage)
        .and_then(|this| NETWORK.load(storage, this.network_id))
        .map_err(|_| ContractError::NetworkConfig)
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct OtherNetwork {
    pub network: NetworkItem,
    pub connection: OtherNetworkItem,
}

pub fn load_other(storage: &dyn Storage, other: NetworkId) -> Result<OtherNetwork> {
    let this = state::load(storage)?;
    let other: NetworkItem = NETWORK.load(storage, other)?;
    let connection = NETWORK_TO_NETWORK.load(storage, (this.network_id, other.network_id))?;
    Ok(OtherNetwork {
        network: other,
        connection : connection.other,
    })
}

pub(crate) fn force_network_to_network(
    _: crate::auth::Auth<crate::auth::policy::Admin>,
    deps: DepsMut,
    msg: cvm_route::transport::NetworkToNetworkItem,
) -> std::result::Result<BatchResponse, crate::error::ContractError> {
    NETWORK_TO_NETWORK.save(deps.storage, (msg.from_network_id, msg.to_network_id), &msg)?;
    if let Some(ibc) = msg.to_other.ics27_channel {
        IBC_CHANNEL_NETWORK.save(deps.storage, ibc.id.to_string(), &msg.to_network_id)?;
    }
    Ok(BatchResponse::new().add_event(
        make_event("network_to_network.forced")
            .add_attribute("to", msg.to_network_id.to_string())
            .add_attribute("from", msg.from_network_id.to_string())
            .add_attribute("ics_20", msg.to_other.ics_20.is_some().to_string()),
    ))
}

pub(crate) fn force_network(
    _auth: crate::auth::Auth<crate::auth::policy::Admin>,
    deps: DepsMut,
    msg: NetworkItem,
) -> crate::error::Result<BatchResponse> {
    NETWORK.save(deps.storage, msg.network_id, &msg)?;
    Ok(BatchResponse::new().add_event(
        make_event("network.forced").add_attribute("network_id", msg.network_id.to_string()),
    ))
}

/// the connection description from first network to second
pub(crate) const NETWORK_TO_NETWORK: Map<(NetworkId, NetworkId), NetworkToNetworkItem> =
    Map::new("network_to_network");

/// network state shared among all networks about it
pub(crate) const NETWORK: Map<NetworkId, NetworkItem> = Map::new("network");


pub fn get_all_networks(deps: Deps) -> StdResult<Vec<NetworkItem>> {
    NETWORK
        .range(deps.storage, None, None, Order::Ascending)
        .map(|x| x.map(|(_, x)| x))
        .collect()
}

pub fn get_all_network_to_network(deps: Deps) -> StdResult<Vec<NetworkToNetworkItem>> {
    NETWORK_TO_NETWORK
        .range(deps.storage, None, None, Order::Ascending)
        .map(|x| x.map(|(_, x)| x))
        .collect()
}