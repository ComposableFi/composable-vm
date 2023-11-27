use crate::{
    gateway::{self, GatewayId, RelativeTimeout},
    prelude::*,
    shared::XcPacket,
    AssetId, NetworkId,
};
use cosmwasm_std::{to_binary, Api, BlockInfo, CosmosMsg, Deps, IbcEndpoint, StdResult, WasmMsg};

use ibc::core::host::types::identifiers::{ChannelId, ConnectionId, PortId};

use ibc_apps::{
    hook::{Callback, IBCLifecycleComplete, SendMemo},
    memo::Memo,
};

/// This message should be send as part of wasm termination memo.
/// So that can match it to sender hash and know what channel and origin was used to send message.
/// All information here is not secured until compared with existing secured data.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct XcMessageData {
    pub from_network_id: NetworkId,
    pub packet: XcPacket,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum TransportTrackerId {
    /// Allows to identify results of IBC packets
    Ibc {
        channel_id: ChannelId,
        sequence: u64,
    },
}

/// route is used to describe how to send a full program packet to another network
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub struct IbcIcs20ProgramRoute {
    pub from_network: NetworkId,
    pub local_native_denom: String,
    pub channel_to_send_over: ChannelId,
    pub sender_gateway: Addr,
    /// the contract address of the gateway to send to assets
    pub gateway_to_send_to: GatewayId,
    pub counterparty_timeout: RelativeTimeout,
    pub ibc_ics_20_sender: IbcIcs20Sender,
    pub on_remote_asset: AssetId,
}

/// send to target chain with cosmwasm receiver
pub fn to_cosmwasm_message<T>(
    _deps: Deps,
    api: &dyn Api,
    coin: Coin,
    route: IbcIcs20ProgramRoute,
    packet: XcPacket,
    block: BlockInfo,
    gateway_to_send_to: Addr,
) -> StdResult<CosmosMsg<T>> {
    let msg = gateway::ExecuteMsg::MessageHook(XcMessageData {
        from_network_id: route.from_network,
        packet,
    });
    let memo = SendMemo {
        inner: Memo {
            wasm: Some(Callback::new_cosmwasm(
                gateway_to_send_to.clone(),
                serde_cw_value::to_value(msg).expect("can always serde"),
            )),

            forward: None,
        },
        ibc_callback: None,
    };
    let memo = serde_json_wasm::to_string(&memo).expect("any memo can be to string");
    api.debug(&format!("cvm::gateway::ibc::ics20::memo {}", &memo));
    match route.ibc_ics_20_sender {
        IbcIcs20Sender::SubstratePrecompile(addr) => {
            unimplemented!("pallet-cosmwasm development was frozen")
        }
        IbcIcs20Sender::CosmosStargateIbcApplicationsTransferV1MsgTransfer => {
            // really
            // https://github.com/osmosis-labs/osmosis-rust/blob/main/packages/osmosis-std-derive/src/lib.rs
            // https://github.com/osmosis-labs/osmosis/blob/main/cosmwasm/packages/registry/src/proto.rs

            use ibc_proto::{
                cosmos::base::v1beta1::Coin, ibc::applications::transfer::v1::MsgTransfer,
            };

            use prost::Message;
            let value = MsgTransfer {
                source_port: PortId::transfer().to_string(),
                source_channel: route.channel_to_send_over.to_string(),
                token: Some(Coin {
                    denom: coin.denom,
                    amount: coin.amount.to_string(),
                }),
                sender: route.sender_gateway.to_string(),
                receiver: gateway_to_send_to.clone().to_string(),
                timeout_height: route
                    .counterparty_timeout
                    .absolute(block.clone())
                    .block()
                    .map(|x| ibc_proto::ibc::core::client::v1::Height {
                        revision_height: x.height,
                        revision_number: x.revision,
                    }),
                timeout_timestamp: route
                    .counterparty_timeout
                    .absolute(block)
                    .timestamp()
                    .map(|x| x.nanos())
                    .unwrap_or_default(),
                memo,
            };
            api.debug(&format!("cvm::gateway::ibc::ics20:: payload {:?}", &value));

            let value = value.encode_to_vec();
            let value = Binary::from(value);

            Ok(CosmosMsg::Stargate {
                type_url: "/ibc.applications.transfer.v1.MsgTransfer".to_string(),
                value,
            })
        }

        IbcIcs20Sender::CosmWasmStd1_3 => Err(cosmwasm_std::StdError::GenericErr {
            msg: "NotSupported".to_string(),
        }),
    }
}

/// Information associated with an IBC channel.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct ChannelInfo {
    /// id of this channel
    pub id: ChannelId,
    /// the remote channel/port we connect to
    pub counterparty_endpoint: IbcEndpoint,
    /// the connection this exists on (you can use to query client/consensus info)
    pub connection_id: ConnectionId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub enum IbcIcs20Sender {
    SubstratePrecompile(Addr),
    CosmosStargateIbcApplicationsTransferV1MsgTransfer,
    CosmWasmStd1_3,
}
