use core::fmt::Display;

use crate::{prelude::*, transport::ForeignAssetId};
use cvm::{AssetId, NetworkId};

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

    /// TODO: make sure one cannot access local if it is bridged until bridged was unwrapped
    /// basically to access asset need to provide network_id to use local
    pub local: AssetReference,
    /// if asset was bridged, it would have way to identify bridge/source/channel
    pub bridged: Option<BridgeAsset>,
}

impl AssetItem {
    pub fn new(asset_id: AssetId, network_id: NetworkId, local: AssetReference) -> Self {
        Self {
            asset_id,
            network_id,
            local,
            bridged: None,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(
    feature = "json-schema", // all(feature = "json-schema", not(target_arch = "wasm32")),
    derive(schemars::JsonSchema)
)]
pub struct NetworkAssetItem {
    pub to_network_id: NetworkId,
    pub from_asset_id: AssetId,
    pub to_asset_id: AssetId,
}

impl NetworkAssetItem {
    pub fn new(to_network_id: NetworkId, from_asset_id: AssetId, to_asset_id: AssetId) -> Self {
        Self {
            to_network_id,
            from_asset_id,
            to_asset_id,
        }
    }
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

pub type SvmPubkey = crate::solana_program::Pubkey;
pub type EvmAddress = crate::primitive_types::H160;

/// Definition of an asset native to some chain to operate on.
/// For example for Cosmos CW and EVM chains both CW20 and ERC20 can be actual.
/// So if asset is local or only remote to some chain depends on context of network or connection.
/// this design leads to some dummy matches, but in general unifies code (so that if one have to
/// solve other chain route it can).
/// One consensus(chain) can have assets produced by different protocols(VMs).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(
    feature = "json-schema", // all(feature = "json-schema", not(target_arch = "wasm32")),
    derive(schemars::JsonSchema)
)]
pub enum AssetReference {
    /// Cosmos SDK native
    Native {
        denom: String,
    },
    Cw20 {
        contract: cosmwasm_std::Addr,
    },

    Erc20 {
        contract: EvmAddress,
    },

    /// Solana VM default token, not only Solana has this token
    SPL20 {
        mint: SvmPubkey,
    },

    /// usually on Polkadot/Kusama and parachains Subtrate runtimes assets encoded as numbers up to u128 value
    PolkadotSubstrateAsset {
        general_index: u128,
    },
}

impl AssetReference {
    pub fn denom(&self) -> String {
        match self {
            AssetReference::Native { denom } => denom.clone(),
            AssetReference::Cw20 { contract } => ["cw20:", contract.as_str()].concat(),
            _ => todo!("implement other asset types"),
        }
    }
}

impl Display for AssetReference {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(&self.denom())
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
            AssetReference::Erc20 { contract } => (2, contract.as_bytes()),
            _ => todo!("implement other asset types"),
        };
        vec![Key::Val8([tag]), Key::Ref(value)]
    }
}
