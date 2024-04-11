//! expresses asset transformation regardless of way transforamtion is done

use crate::prelude::*;
use cvm::{exchange::ExchangeId, AssetId};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[cfg_attr(
    feature = "json-schema", // all(feature = "json-schema", not(target_arch = "wasm32")),
    derive(schemars::JsonSchema)
)]
#[serde(rename_all = "snake_case")]
pub enum VenueId {
    Exchange(ExchangeId),
    Transfer,
}

/// assets which can be transomed into each other via venue
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[cfg_attr(
    feature = "json-schema", // all(feature = "json-schema", not(target_arch = "wasm32")),
    derive(schemars::JsonSchema)
)]
#[serde(rename_all = "snake_case")]
pub struct AssetsVenueItem {
    pub venue_id: VenueId,
    pub from_asset_id: AssetId,
    pub to_asset_id: AssetId,
}

impl AssetsVenueItem {
    pub fn new(venue_id: VenueId, from_asset_id: AssetId, to_asset_id: AssetId) -> Self {
        Self {
            venue_id,
            from_asset_id,
            to_asset_id,
        }
    }
}