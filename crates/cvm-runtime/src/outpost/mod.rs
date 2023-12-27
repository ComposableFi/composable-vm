pub mod config;
mod query;

use alloc::fmt::format;
pub use config::*;
use cosmwasm_std::{ensure, StdError};
use cvm_route::{
    asset::{AssetItem, AssetReference},
    exchange::ExchangeItem,
};
pub use query::*;

use crate::{
    exchange::*, prelude::*, transport::ibc::XcMessageData, AssetId, CallOrigin, ExecutorOrigin,
    Funds, NetworkId,
};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(
    feature = "json-schema", // all(feature = "json-schema", not(target_arch = "wasm32")),
    derive(schemars::JsonSchema)
)]
pub struct MigrateMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, derive_more::From)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(
    feature = "json-schema", // all(feature = "json-schema", not(target_arch = "wasm32")),
    derive(schemars::JsonSchema)
)]
pub enum ExecuteMsg {
    Config(ConfigSubMsg),
    Admin(AdminSubMsg),

    /// Sent by the user to execute a program on their behalf.
    ExecuteProgram(ExecuteProgramMsg),

    /// Request to execute a program on behalf of given user.
    ///
    /// This can only be sent by trusted contract.  The message is
    ExecuteProgramPrivileged {
        /// The origin of the call.
        call_origin: CallOrigin,
        /// Program to execute.
        execute_program: BridgeExecuteProgramMsg,
    },

    /// Message sent from executor trying to spawn program on another
    /// network.
    BridgeForward(BridgeForwardMsg),

    /// simple permissionless message which produce xcvm program to test routes
    Shortcut(ShortcutSubMsg),

    /// executed by host as part of memo handling
    MessageHook(XcMessageData),
}

/// can only be executed by gov or admin
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(
    feature = "json-schema", // all(feature = "json-schema", not(target_arch = "wasm32")),
    derive(schemars::JsonSchema)
)]
pub enum AdminSubMsg {
    ExecutePacketICS20(ExecutePacketICS20Msg),
}

/// can only be executed by gov or admin
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(
    feature = "json-schema", // all(feature = "json-schema", not(target_arch = "wasm32")),
    derive(schemars::JsonSchema)
)]
pub struct ExecutePacketICS20Msg {
    #[serde(
        serialize_with = "hex::serialize",
        deserialize_with = "hex::deserialize"
    )]
    pub packet_data_hex: Vec<u8>,
}

impl ExecutePacketICS20Msg {
    pub fn into_packet(self) -> Result<ibc_app_transfer_types::packet::PacketData, StdError> {
        let ics20 = serde_json_wasm::from_slice(&self.packet_data_hex)
            .map_err(|x| StdError::generic_err(format!("{:?}", x)))?;
        Ok(ics20)
    }

    pub fn into_wasm_hook(self, contract: ibc_primitives::Signer) -> Result<ExecuteMsg, StdError> {
        let ics20 = self.into_packet()?;
        let memo: ibc_apps_more::memo::Memo = serde_json_wasm::from_str(&ics20.memo.to_string())
            .map_err(|x| StdError::generic_err(format!("{:?}", x)))?;
        let wasm: ibc_apps_more::hook::Callback = memo
            .wasm
            .ok_or(StdError::generic_err(format!("no wasm in memo")))?;
        ensure!(
            wasm.contract == contract,
            StdError::generic_err(format!("wasm contract mismatch {}", wasm.contract))
        );
        let hook: ExecuteMsg = wasm
            .msg
            .deserialize_into()
            .map_err(|x| StdError::generic_err(format!("{:?}", x)))?;
        Ok(hook)
    }
}

#[cfg(test)]
pub mod test {
    use ibc_primitives::Signer;

    use super::ExecutePacketICS20Msg;

    #[test]
    pub fn decode() {
        let example = r#"{"packet_data_hex": "7b22616d6f756e74223a22313030303030222c2264656e6f6d223a227472616e736665722f6368616e6e656c2d31382f756e74726e222c226d656d6f223a227b5c227761736d5c223a7b5c22636f6e74726163745c223a5c226e657574726f6e31737567717a66736e3466753761706361683538657a32346a72663765776d3678326330737a306a7776706b64756b3464733838736c616e3532775c222c5c226d73675c223a7b5c226d6573736167655f686f6f6b5c223a7b5c2266726f6d5f6e6574776f726b5f69645c223a322c5c227061636b65745c223a7b5c226173736574735c223a5b5b5c223135383435363332353032383532383637353138373038373930303637375c222c5c223130303030305c225d5d2c5c226578656375746f725c223a5c2236333635366537343631373537323639333136373332376136313662363136633635373337373636373537343638373633333664373536373335366237363730363336653339373037353634376137313661373633363738333333363631373033303664363436353338333437303661373736653636373637333739373737353337363336375c222c5c2270726f6772616d5c223a7b5c22696e737472756374696f6e735c223a5b5d7d2c5c2273616c745c223a5c225c222c5c22757365725f6f726967696e5c223a7b5c226e6574776f726b5f69645c223a322c5c22757365725f69645c223a5c22363336353665373436313735373236393331373533323733373233303730333236613337333536363735363537613735333933323665363637383637333537373664333433363637373533323332373937373636363737353663333636625c227d7d7d7d7d7d222c227265636569766572223a226e657574726f6e31737567717a66736e3466753761706361683538657a32346a72663765776d3678326330737a306a7776706b64756b3464733838736c616e353277222c2273656e646572223a2263656e746175726931766c68366b6e79783837306b326d63396673707830386b613466756464737878367338746a7038336e37786b79636b6b676d387132356d777236227d"}"#;
        let parsed: ExecutePacketICS20Msg = serde_json_wasm::from_str(example).unwrap();
        let hook = parsed
            .into_wasm_hook(
                ("neutron1sugqzfsn4fu7apcah58ez24jrf7ewm6x2c0sz0jwvpkduk4ds88slan52w"
                    .to_string()
                    .into()),
            )
            .unwrap();
    }
}

//
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(
    feature = "json-schema", // all(feature = "json-schema", not(target_arch = "wasm32")),
    derive(schemars::JsonSchema)
)]
pub enum ShortcutSubMsg {
    Transfer {
        /// assets from there
        asset_id: AssetId,
        amount: Uint128,
        /// target network, can hope over several networks
        /// if route is stored in state
        network: NetworkId,
        /// by default receiver is this
        receiver: Option<String>,
    },
}

/// Definition of a program to be executed including its context.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(
    feature = "json-schema", // all(feature = "json-schema", not(target_arch = "wasm32")),
    derive(schemars::JsonSchema)
)]
pub struct ExecuteProgramMsg<Assets = Option<Funds<crate::shared::Displayed<u128>>>> {
    /// The program salt.
    /// If JSON, than hex encoded non prefixed lower case string.
    /// If not specified, uses no salt.
    #[serde(
        serialize_with = "hex::serialize",
        deserialize_with = "hex::deserialize"
    )]
    #[cfg_attr(
        feature = "json-schema", // all(feature = "json-schema", not(target_arch = "wasm32")),
        schemars(schema_with = "String::json_schema")
    )]
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub salt: Vec<u8>,
    /// The program.
    pub program: crate::shared::XcProgram,
    /// Assets to fund the CVM executor instance.
    /// The executor is funded prior to execution.
    /// If None, 100% of received funds go to executor.
    pub assets: Assets,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub tip: Option<String>,
}

/// message sent within CVM must have assets defined
pub type BridgeExecuteProgramMsg = ExecuteProgramMsg<Funds<crate::shared::Displayed<u128>>>;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(
    feature = "json-schema", // all(feature = "json-schema", not(target_arch = "wasm32")),
    derive(schemars::JsonSchema)
)]
pub struct BridgeForwardMsg {
    pub executor_origin: ExecutorOrigin,
    /// target network
    pub to: NetworkId,
    pub msg: BridgeExecuteProgramMsg,
}

/// Wrapper for interfacing with a gateway contract.
///
/// Provides convenience methods for querying the gateway and sending execute
/// messages to it.  Queries use [`cosmwasm_std::QuerierWrapper`] to make the
/// request and return immediately.  Execute requests on the other hand are
/// asynchronous and done by returning a [`cosmwasm_std::CosmosMsg`] which needs
/// to be added to a [`cosmwasm_std::Response`] object.
///
/// The object can be JSON-serialised as the address of the gateway.  Note that
/// since it’s serialised as [`cosmwasm_std::Addr`] it should not be part of
/// public API and only serialised in trusted objects where addresses don’t need
/// to be validated.
#[cfg(feature = "cosmwasm")]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(
    feature = "json-schema", // all(feature = "json-schema", not(target_arch = "wasm32")),
    derive(schemars::JsonSchema)
)]
#[serde(transparent)]
pub struct Outpost {
    address: cosmwasm_std::Addr,
}

#[cfg(feature = "cosmwasm")]
impl Outpost {
    pub fn new(address: cosmwasm_std::Addr) -> Self {
        Self { address }
    }

    /// Validates gateway address and if it’s correct constructs a new object.
    ///
    /// This is mostly a wrapper around CosmWasm address validation API.
    pub fn addr_validate(
        api: &dyn cosmwasm_std::Api,
        address: &str,
    ) -> cosmwasm_std::StdResult<Self> {
        api.addr_validate(address).map(Self::new)
    }

    /// Returns gateway contract’s address as a String.
    pub fn address(&self) -> cosmwasm_std::Addr {
        self.address.clone()
    }

    /// Creates a CosmWasm message executing given message on the gateway.
    ///
    /// The returned message must be added to Response to take effect.
    pub fn execute(
        &self,
        msg: impl Into<ExecuteMsg>,
    ) -> cosmwasm_std::StdResult<cosmwasm_std::CosmosMsg> {
        self.execute_with_funds(msg, Vec::new())
    }

    /// Creates a CosmWasm message executing given message on the gateway with
    /// given funds attached.
    ///
    /// The returned message must be added to Response to take effect.
    pub fn execute_with_funds(
        &self,
        msg: impl Into<ExecuteMsg>,
        funds: Vec<cosmwasm_std::Coin>,
    ) -> cosmwasm_std::StdResult<cosmwasm_std::CosmosMsg> {
        cosmwasm_std::wasm_execute(self.address(), &msg.into(), funds)
            .map(cosmwasm_std::CosmosMsg::from)
    }

    /// Queries the gateway for definition of an asset with given id.
    pub fn get_asset_by_id(
        &self,
        querier: cosmwasm_std::QuerierWrapper,
        asset_id: AssetId,
    ) -> cosmwasm_std::StdResult<AssetItem> {
        let query = QueryMsg::GetAssetById { asset_id };
        self.do_query::<GetAssetResponse>(querier, query)
            .map(|response| response.asset)
    }

    pub fn get_exchange_by_id(
        &self,
        querier: cosmwasm_std::QuerierWrapper,
        exchange_id: ExchangeId,
    ) -> cosmwasm_std::StdResult<ExchangeItem> {
        let query = QueryMsg::GetExchangeById { exchange_id };
        self.do_query::<GetExchangeResponse>(querier, query)
            .map(|response| response.exchange)
    }

    /// Queries the gateway for definition of an asset with given local
    /// reference.
    pub fn get_local_asset_by_reference(
        &self,
        querier: cosmwasm_std::QuerierWrapper,
        reference: AssetReference,
    ) -> cosmwasm_std::StdResult<AssetItem> {
        let query = QueryMsg::GetLocalAssetByReference { reference };
        self.do_query::<GetAssetResponse>(querier, query)
            .map(|response| response.asset)
    }

    /// Sends a query to the gateway contract.
    fn do_query<R: serde::de::DeserializeOwned>(
        &self,
        querier: cosmwasm_std::QuerierWrapper,
        query: QueryMsg,
    ) -> cosmwasm_std::StdResult<R> {
        let query = cosmwasm_std::WasmQuery::Smart {
            contract_addr: self.address().into(),
            msg: cosmwasm_std::to_json_binary(&query)?,
        };
        querier.query(&query.into())
    }
}
