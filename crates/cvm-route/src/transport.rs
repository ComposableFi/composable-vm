use crate::prelude::*;
use cosmwasm_std::{BlockInfo, IbcTimeout};
use cvm::NetworkId;
use ibc_app_transfer_types::PrefixedDenom;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(
    feature = "json-schema", // all(feature = "json-schema", not(target_arch = "wasm32")),
    derive(schemars::JsonSchema)
)]
pub struct NetworkToNetwork {
    pub from: NetworkId,
    pub to: NetworkId,

    /// on `to` chain
    pub other: OtherNetworkItem,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(
    feature = "json-schema", // all(feature = "json-schema", not(target_arch = "wasm32")),
    derive(schemars::JsonSchema)
)]
pub struct OtherNetworkItem {
    pub ics_20: Option<IcsPair>,
    /// default timeout to use for direct send
    pub counterparty_timeout: RelativeTimeout,
    /// if there is ICS27 IBC channel opened
    pub ics27_channel: Option<ChannelInfo>,
    /// if true, than will use shortcuts
    /// for example,
    /// if program transfer only program will just use native transfer
    /// or if connection supports exchange, it will use exchange
    /// default is false if target chain has CVM gateway
    pub use_shortcut: Option<bool>,
}

/// we need both, so we can unwrap
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(
    feature = "json-schema", // all(feature = "json-schema", not(target_arch = "wasm32")),
    derive(schemars::JsonSchema)
)]
pub struct IcsPair {
    pub source: ibc_core_host_types::identifiers::ChannelId,
    pub sink: ibc_core_host_types::identifiers::ChannelId,
}

/// relative timeout to CW/IBC-rs time.
/// very small, assumed messages are arriving fast enough, like less than hours
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(
    feature = "json-schema", // all(feature = "json-schema", not(target_arch = "wasm32")),
    derive(schemars::JsonSchema)
)]
pub enum RelativeTimeout {
    /// Timeout is relative to the current block timestamp of counter party
    Seconds(u16),
}

impl RelativeTimeout {
    #[cfg(feature = "cosmwasm")]
    pub fn absolute(&self, block: BlockInfo) -> IbcTimeout {
        match self {
            RelativeTimeout::Seconds(seconds) => {
                IbcTimeout::with_timestamp(block.time.plus_seconds(*seconds as u64))
            }
        }
    }
}

/// Information associated with an IBC channel.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct ChannelInfo {
    /// id of this channel
    pub id: ibc_core_host_types::identifiers::ChannelId,
    /// the remote channel/port we connect to
    pub counterparty_endpoint: cosmwasm_std::IbcEndpoint,
    /// the connection this exists on (you can use to query client/consensus info)
    pub connection_id: ibc_core_host_types::identifiers::ConnectionId,
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(
    all(
        feature = "json-schema", // all(feature = "json-schema", not(target_arch = "wasm32")),
        not(feature = "xcm")
    ),
    derive(schemars::JsonSchema)
)]
pub enum ForeignAssetId {
    IbcIcs20(PrefixedDenom),
    #[cfg(feature = "xcm")]
    Xcm(xcm::VersionedMultiLocation),
}

#[cfg(feature = "xcm")]
impl parity_scale_codec::MaxEncodedLen for ForeignAssetId {
    fn max_encoded_len() -> usize {
        2048
    }
}

#[cfg(feature = "xcm")]
impl From<xcm::VersionedMultiLocation> for ForeignAssetId {
    fn from(this: xcm::VersionedMultiLocation) -> Self {
        Self::Xcm(this)
    }
}

impl From<PrefixedDenom> for ForeignAssetId {
    fn from(this: PrefixedDenom) -> Self {
        Self::IbcIcs20(this)
    }
}
