use cvm_route::transport::ChannelInfo;
use cw_storage_plus::Map;

use cvm_runtime::NetworkId;

pub(crate) const IBC_CHANNEL_NETWORK: Map<String, NetworkId> = Map::new("ibc_channel_network");
pub(crate) const IBC_CHANNEL_INFO: Map<String, ChannelInfo> = Map::new("ibc_channel_info");
