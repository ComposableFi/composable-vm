use crate::{
	prelude::*,
	service::dex::{ExchangeId, ExchangeItem},
	AssetId, NetworkId,
};

use super::{AssetItem, AssetReference};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema, QueryResponses))]
pub enum QueryMsg {
	/// Returns [`AssetReference`] for an asset with given id.
	#[cfg_attr(feature = "json-schema", returns(GetAssetResponse))]
	GetAssetById { asset_id: AssetId },

	/// Returns [`AssetItem`] for an asset with given local reference.
	#[cfg_attr(feature = "json-schema", returns(GetAssetResponse))]
	GetLocalAssetByReference { reference: AssetReference },

	#[cfg_attr(feature = "json-schema", returns(GetIbcIcs20RouteResponse))]
	GetIbcIcs20Route { to_network: NetworkId, for_asset: AssetId },

	#[cfg_attr(feature = "json-schema", returns(GetExchangeResponse))]
	GetExchangeById { exchange_id: ExchangeId },
}

/// gets all assets in CVM registry without underlying native information
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct GetAllAssetsResponse {
	pub assets: Vec<AssetId>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct GetExchangeResponse {
	pub exchange: ExchangeItem,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct GetIbcIcs20RouteResponse {
	pub route: crate::transport::ibc::IbcIcs20ProgramRoute,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct GetAssetResponse {
	pub asset: AssetItem,
}
