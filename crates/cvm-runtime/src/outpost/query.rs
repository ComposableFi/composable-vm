use crate::{prelude::*, AssetId, ExchangeId, NetworkId};

use cvm_route::{
    asset::{AssetItem, AssetReference, NetworkAssetItem},
    exchange::ExchangeItem,
    transport::NetworkToNetworkItem,
    *,
};

use super::NetworkItem;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(
    feature = "json-schema", // all(feature = "json-schema", not(target_arch = "wasm32")),
    derive(schemars::JsonSchema, cosmwasm_schema::QueryResponses)
)]
pub enum QueryMsg {
    /// Returns [`AssetReference`] for an asset with given id.
    #[cfg_attr(
        feature = "json-schema", // all(feature = "json-schema", not(target_arch = "wasm32")),
        returns(GetAssetResponse)
    )]
    GetAssetById { asset_id: AssetId },

    /// Returns [`AssetItem`] for an asset with given local reference.
    #[cfg_attr(
        feature = "json-schema", // all(feature = "json-schema", not(target_arch = "wasm32")),
        returns(GetAssetResponse)
    )]
    GetLocalAssetByReference { reference: AssetReference },

    #[cfg_attr(
        feature = "json-schema", // all(feature = "json-schema", not(target_arch = "wasm32")),
        returns(GetIbcIcs20RouteResponse)
    )]
    GetIbcIcs20Route {
        to_network: NetworkId,
        for_asset: AssetId,
    },

    #[cfg_attr(
        feature = "json-schema", // all(feature = "json-schema", not(target_arch = "wasm32")),
        returns(GetExchangeResponse)
    )]
    GetExchangeById { exchange_id: ExchangeId },
    /// Get all information this CVM knows about local and foreign assets/exchanges/networks
    #[cfg_attr(
        feature = "json-schema", // all(feature = "json-schema", not(target_arch = "wasm32")),
        returns(GetConfigResponse)
    )]
    GetConfig {},
    // /// So given program, contract returns route which will follow
    // #[cfg_attr(
    //     feature = "json-schema", // all(feature = "json-schema", not(target_arch = "wasm32")),
    //     returns(GetRouteResponse)
    // )]
    // GetRoute { program: crate::shared::XcProgram },
}

/// gets all assets in CVM registry without underlying native information
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(
    feature = "json-schema", // all(feature = "json-schema", not(target_arch = "wasm32")),
    derive(schemars::JsonSchema)
)]
pub struct GetAllAssetsResponse {
    pub assets: Vec<AssetId>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(
    feature = "json-schema", // all(feature = "json-schema", not(target_arch = "wasm32")),
    derive(schemars::JsonSchema)
)]
pub struct GetExchangeResponse {
    pub exchange: ExchangeItem,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(
    feature = "json-schema", // all(feature = "json-schema", not(target_arch = "wasm32")),
    derive(schemars::JsonSchema)
)]
pub struct GetIbcIcs20RouteResponse {
    pub route: crate::transport::ibc::IbcIcs20ProgramRoute,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(
    feature = "json-schema", // all(feature = "json-schema", not(target_arch = "wasm32")),
    derive(schemars::JsonSchema)
)]
pub struct GetAssetResponse {
    pub asset: AssetItem,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(
    feature = "json-schema", // all(feature = "json-schema", not(target_arch = "wasm32")),
    derive(schemars::JsonSchema)
)]
pub struct GetConfigResponse {
    pub network_to_networks: Vec<NetworkToNetworkItem>,
    pub assets: Vec<AssetItem>,
    pub exchanges: Vec<ExchangeItem>,
    pub networks: Vec<NetworkItem>,
    pub network_assets: Vec<NetworkAssetItem>,
}

impl GetConfigResponse {
    pub fn get_network_for_asset(&self, asset_id: AssetId) -> NetworkId {
        self.assets
            .iter()
            .find(|x| x.asset_id == asset_id)
            .map(|x| x.network_id)
            .expect("network_id")
    }
}
