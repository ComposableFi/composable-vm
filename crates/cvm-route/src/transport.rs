#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(
    feature = "json-schema", // all(feature = "json-schema", not(target_arch = "wasm32")),
    derive(schemars::JsonSchema)
)]
pub struct ForceNetworkToNetworkMsg {
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
    pub source: ChannelId,
    pub sink: ChannelId,
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
    pub fn absolute(&self, block: BlockInfo) -> IbcTimeout {
        match self {
            RelativeTimeout::Seconds(seconds) => {
                IbcTimeout::with_timestamp(block.time.plus_seconds(*seconds as u64))
            }
        }
    }
}
