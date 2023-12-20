use crate::{prelude::*, transport::ForeignAssetId};
use cvm::{AssetId, NetworkId};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(
    feature = "json-schema", // all(feature = "json-schema", not(target_arch = "wasm32")),
    derive(schemars::JsonSchema)
)]
pub struct AssetToNetwork {
    pub this_asset: AssetId,
    pub other_network: NetworkId,
    pub other_asset: AssetId,
}

impl AssetToNetwork {
    pub fn new(this_asset: AssetId, other_network: NetworkId, other_asset: AssetId) -> Self {
        Self {
            this_asset,
            other_network,
            other_asset,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(
    feature = "json-schema", // all(feature = "json-schema", not(target_arch = "wasm32")),
    derive(schemars::JsonSchema)
)]
pub struct AssetItem {
    pub asset_id: AssetId,
    /// network id on which this asset id can be used locally
    pub network_id: NetworkId,
    pub local: AssetReference,
    /// if asset was bridged, it would have way to identify bridge/source/channel
    pub bridged: Option<BridgeAsset>,
}

impl AssetItem {
    pub fn denom(&self) -> String {
        self.local.denom()
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(
    feature = "json-schema", // all(feature = "json-schema", not(target_arch = "wasm32")),
    derive(schemars::JsonSchema)
)]
pub struct BridgeAsset {
    pub location_on_network: ForeignAssetId,
}

/// Definition of an asset native to some chain to operate on.
/// For example for Cosmos CW and EVM chains both CW20 and ERC20 can be actual.
/// So if asset is local or only remote to some chain depends on context of network or connection.
/// this design leads to some dummy matches, but in general unifies code (so that if one have to
/// solve other chain route it can)
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(
    feature = "json-schema", // all(feature = "json-schema", not(target_arch = "wasm32")),
    derive(schemars::JsonSchema)
)]
pub enum AssetReference {
    Native { denom: String },
    Cw20 { contract: cosmwasm_std::Addr },
    // Erc20 { contract: EthAddress },
}

impl AssetReference {
    pub fn denom(&self) -> String {
        match self {
            AssetReference::Native { denom } => denom.clone(),
            AssetReference::Cw20 { contract } => ["cw20:", contract.as_str()].concat(),
            //AssetReference::Erc20 { contract } => ["erc20:", &contract.to_string()].concat(),
        }
    }
}

#[cfg(feature = "cosmwasm")]
impl cw_storage_plus::PrimaryKey<'_> for AssetReference {
    type Prefix = ();
    type SubPrefix = ();
    type Suffix = ();
    type SuperSuffix = ();

    #[inline]
    fn key(&self) -> Vec<cw_storage_plus::Key<'_>> {
        use cw_storage_plus::Key;
        let (tag, value) = match self {
            AssetReference::Native { denom } => (0, denom.as_bytes()),
            AssetReference::Cw20 { contract } => (1, contract.as_bytes()),
            // AssetReference::Erc20 { contract } => (2, contract.as_bytes()),
        };
        vec![Key::Val8([tag]), Key::Ref(value)]
    }
}
