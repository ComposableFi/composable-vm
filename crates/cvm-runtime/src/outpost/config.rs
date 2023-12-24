use cosmwasm_std::{BlockInfo, IbcTimeout};
use cvm_route::{
    asset::AssetToNetwork, exchange::ExchangeItem, transport::NetworkToNetworkItem, *,
};
use ibc_core_host_types::identifiers::ChannelId;

use crate::{prelude::*, transport::ibc::IbcEnabled, AssetId, NetworkId};

type EthAddress = [u8; 20]; // primitive_types::H160;

/// Version of IBC channels used by the gateway.
pub const IBC_VERSION: &str = "xcvm-v0";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(
    feature = "json-schema", // all(feature = "json-schema", not(target_arch = "wasm32")),
    derive(schemars::JsonSchema)
)]
pub struct OsmosisIbcHooks {
    pub callback: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(
    feature = "json-schema", // all(feature = "json-schema", not(target_arch = "wasm32")),
    derive(schemars::JsonSchema)
)]
pub struct PFM {}

/// if chain has IBC SDK callbacks enabled
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(
    feature = "json-schema", // all(feature = "json-schema", not(target_arch = "wasm32")),
    derive(schemars::JsonSchema)
)]
pub struct Adr08IbcCallbacks {}

/// what features/modules/version enabled/installed/configured
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(
    feature = "json-schema", // all(feature = "json-schema", not(target_arch = "wasm32")),
    derive(schemars::JsonSchema)
)]
pub struct Ics20Features {
    /// if it is exists, chain has that enabled
    pub wasm_hooks: Option<OsmosisIbcHooks>,
    pub ibc_callbacks: Option<Adr08IbcCallbacks>,
    pub pfm: Option<PFM>,
}

/// given prefix you may form accounts from 32 bit addresses or partially identify chains
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(
    feature = "json-schema", // all(feature = "json-schema", not(target_arch = "wasm32")),
    derive(schemars::JsonSchema)
)]
pub enum Prefix {
    SS58(u16),
    Bech(String),
    // no prefix, pure Ethereum EVM
    // EthEvm,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(
    feature = "json-schema", // all(feature = "json-schema", not(target_arch = "wasm32")),
    derive(schemars::JsonSchema)
)]
pub struct NetworkItem {
    pub network_id: NetworkId,
    /// something which will be receiver on other side
    /// case of network has XCVM deployed as contract, account address is stored here
    pub outpost: Option<OutpostId>,
    /// Account encoding type
    pub accounts: Option<Prefix>,
    pub ibc: Option<IbcEnabled>,
}

/// cross cross chain routing requires a lot of configuration,
/// about chain executing this contract,
/// about connectivity to and of other chains (even if not connected directly)
/// and about assets and services on these chains
/// (in future block hooks and some set of host extensions/precompiles would help to get some info
/// automatically)
/// `Force` message sets the data unconditionally.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(
    feature = "json-schema", // all(feature = "json-schema", not(target_arch = "wasm32")),
    derive(schemars::JsonSchema)
)]
pub enum ConfigSubMsg {
    /// Permissioned message (gov or admin) to force set information about network contract is
    /// executed. Network can be any network or this network (so it overrides some this network
    /// parameters too)
    ForceNetwork(NetworkItem),
    /// Sets network to network connectivity/routing information
    ForceNetworkToNetwork(NetworkToNetworkItem),

    /// Permissioned message (gov or admin) to force set asset information.
    ForceAsset(cvm_route::asset::AssetItem),

    ForceAssetToNetworkMap(AssetToNetwork),

    ForceExchange(ExchangeItem),

    /// Message sent by an admin to remove an asset from registry.
    ForceRemoveAsset {
        asset_id: AssetId,
    },

    // https://github.com/CosmWasm/cosmwasm/discussions/1814
    /// short cut to rollout config faster
    Force(Vec<ConfigSubMsg>),

    /// instantiates default executor on behalf of user
    /// `salt` - human string, converted to hex or base64 depending on implementation
    ForceInstantiate {
        user_origin: Addr,
        #[serde(skip_serializing_if = "String::is_empty", default)]
        salt: String,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(
    feature = "json-schema", // all(feature = "json-schema", not(target_arch = "wasm32")),
    derive(schemars::JsonSchema)
)]
pub struct InstantiateMsg(pub HereItem);

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(
    feature = "json-schema", // all(feature = "json-schema", not(target_arch = "wasm32")),
    derive(schemars::JsonSchema)
)]
pub struct HereItem {
    /// Network ID of this network where contract is deployed
    pub network_id: NetworkId,
    /// The admin which is allowed to update the bridge list.
    pub admin: Addr,
}

/// when message is sent to other side, we should identify receiver of some kind
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(
    feature = "json-schema", // all(feature = "json-schema", not(target_arch = "wasm32")),
    derive(schemars::JsonSchema)
)]
pub enum OutpostId {
    CosmWasm {
        contract: Addr,
        /// CVM executor contract code
        executor_code_id: u64,
        /// admin of everything
        admin: Addr,
    },
    // Evm {
    //     contract: EthAddress,
    //     admin: EthAddress,
    // },
}
