//! Helps connecting identifiers into networks.
//! Allows to map asset identifiers, contracts, networks, channels, denominations from, to and on
//! each chain via contract storage, precompiles, host extensions.
//! handles PFM and IBC wasm hooks
use crate::{
    contract::ReplyId,
    prelude::*,
    state::network::{self, load_this},
};
use cosmwasm_std::{
    ensure_eq, wasm_execute, Binary, BlockInfo, Coin, Deps, DepsMut, Env, MessageInfo, Response,
    Storage, SubMsg,
};
use cvm_route::asset::{AssetItem, AssetReference};
use cvm_runtime::{
    outpost::{ExecuteMsg, ExecuteProgramMsg, OutpostId},
    shared::{XcFunds, XcPacket, XcProgram},
    transport::ibc::{to_cosmwasm_message, IbcIcs20ProgramRoute, XcMessageData},
    AssetId, CallOrigin,
};

use crate::{
    auth,
    error::{ContractError, Result},
    events::make_event,
};

// 1. if there is know short cat multi hop path it is used up to point in cannot be used anymore (in
// this case CVM Executor call is propagated) 2. if there is no solved multiprop route, only single
// hope route checked amid 2 networks and if can do shortcut 3. else full CVM Executor call is
// propagated
pub(crate) fn handle_bridge_forward(
    _: auth::Executor,
    deps: DepsMut,
    info: MessageInfo,
    msg: cvm_runtime::outpost::BridgeForwardMsg,
    block: BlockInfo,
) -> Result {
    deps.api.debug(&format!(
        "cvm::outpost::bridge::forward::ibc::ics20::memo {}",
        &serde_json_wasm::to_string(&msg)?
    ));

    ensure_eq!(
        msg.msg.assets.0.len(),
        1,
        ContractError::ProgramCannotBeHandledByDestination
    );
    let (local_asset, amount) = msg.msg.assets.0.first().expect("proved above");

    let (msg, event) = if let Ok(transfer_shortcut) =
        ibc_ics_20_transfer_shortcut(deps.as_ref(), &msg)
    {
        let mut _event = make_event("bridge")
            .add_attribute("to_network_id", msg.to_network.to_string())
            .add_attribute("shortcut", "ics20-transfer");

        unimplemented!("add tracking lock for funds return usual cosmos message to transfer as defined in {:?}", transfer_shortcut);
    } else {
        let route: IbcIcs20ProgramRoute = get_this_route(deps.storage, msg.to_network, *local_asset)?;
        crate::state::tracking::bridge_lock(deps.storage, (msg.clone(), route.clone()))?;

        let asset = msg
            .msg
            .assets
            .0
            .first()
            .map(|(_, amount)| (route.on_remote_asset, *amount))
            .expect("not empty");

        let packet = XcPacket {
            executor: String::from(info.sender).into_bytes(),
            user_origin: msg.executor_origin.user_origin.clone(),
            salt: msg.msg.salt,
            program: msg.msg.program,
            assets: vec![asset].into(),
        };

        deps.api.debug(&format!(
            "cvm::outpost::ibc::ics20 route {}",
            &serde_json_wasm::to_string(&route)?
        ));

        let mut event = make_event("bridge")
            .add_attribute("to_network_id", msg.to_network.to_string())
            .add_attribute(
                "assets",
                serde_json_wasm::to_string(&packet.assets)
                    .map_err(|_| ContractError::FailedToSerialize)?,
            )
            .add_attribute(
                "program",
                serde_json_wasm::to_string(&packet.program)
                    .map_err(|_| ContractError::FailedToSerialize)?,
            );

        if !packet.salt.is_empty() {
            let salt_attr = Binary::from(packet.salt.as_slice()).to_string();
            event = event.add_attribute("salt", salt_attr);
        }

        let coin = Coin::new(amount.0, route.local_native_denom.clone());

        match route.to_outpost.clone() {
            OutpostId::CosmWasm { contract, .. } => {
                let msg = to_cosmwasm_message(
                    deps.as_ref(),
                    deps.api,
                    coin,
                    route,
                    packet,
                    block,
                    contract,
                )?;
                (msg, event)
            } // OutpostId::Evm { .. } => Err(ContractError::NotImplemented)?,
        }
    };

    Ok(Response::default()
        .add_event(event)
        .add_submessage(SubMsg::reply_on_success(msg, ReplyId::TransportSent.into())))
}

/// When target network supports native cross chain operation of Transfer,
/// and program as simple as just Transfer,
/// can use instance of this structure to route pure funds transfer
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[cfg_attr(
    feature = "json-schema", // all(feature = "json-schema", not(target_arch = "wasm32")),
    derive(schemars::JsonSchema)
)]
#[serde(rename_all = "snake_case")]
pub struct IbcIcs20TransferShortcutRoute {
    pub source: ChannelId,
    pub denom: String,
    pub sending: cvm_runtime::transport::ibc::IbcIcs20Sender,
}

/// this method return route in case program can be just transfer
pub fn ibc_ics_20_transfer_shortcut(
    deps: Deps,
    msg: &cvm_runtime::outpost::BridgeForwardMsg,
) -> Result<IbcIcs20TransferShortcutRoute, ContractError> {
    let storage = deps.storage;
    let this = load_this(storage)?;
    let other = network::load_other(storage, msg.to_network)?;
    let this_asset_id = msg.msg.assets.0[0].0;
    let asset: AssetItem = crate::state::assets::ASSETS
        .load(storage, this_asset_id)
        .map_err(|_| ContractError::AssetNotFoundById(this_asset_id))?;
    if let Some(ics20) = other.connection.ics_20 {
        if let Some(shortcut) = other.connection.use_shortcut {
            if shortcut {
                return Ok(IbcIcs20TransferShortcutRoute {
                    source: ics20.source,
                    denom: asset.local.denom(),
                    sending: this
                        .ibc
                        .expect("ibc")
                        .channels
                        .expect("channels")
                        .ics20
                        .expect("ics20")
                        .sender,
                });
            }
        }
    }
    Err(ContractError::ICS20NotFound)
}

/// given target network and this network assets identifier,
/// find channels, target denomination and gateway on other network
/// so can form and sent ICS20 PFM Wasm terminated packet
/// starts on this network only
pub fn get_this_route(
    storage: &dyn Storage,
    to_network_id: cvm_runtime::NetworkId,
    this_asset_id: AssetId,
) -> Result<IbcIcs20ProgramRoute, ContractError> {
    let this = load_this(storage)?;
    let other = network::load_other(storage, to_network_id)?;
    let asset: AssetItem = crate::state::assets::ASSETS
        .load(storage, this_asset_id)
        .map_err(|_| ContractError::AssetNotFoundById(this_asset_id))?;
    let to_asset: AssetId = crate::state::assets::NETWORK_ASSET
        .load(storage, (to_network_id, this_asset_id))
        .map_err(|_| {
            ContractError::AssetCannotBeTransferredToNetwork(this_asset_id, to_network_id)
        })?
        .asset_id;
    let to_outpost = other
        .network
        .outpost
        .ok_or(ContractError::UnsupportedNetwork)?;

    let from_outpost = match this.outpost.expect("we execute here") {
        OutpostId::CosmWasm { contract, .. } => contract,
        // OutpostId::Evm { .. } => {
        //     Err(ContractError::BadlyConfiguredRouteBecauseThisChainCanSendOnlyFromCosmwasm)?
        // }
    };

    let channel = other
        .connection
        .ics_20
        .ok_or(ContractError::ICS20NotFound)?
        .source;

    Ok(IbcIcs20ProgramRoute {
        from_network: this.network_id,
        local_native_denom: asset.local.denom(),
        channel_to_send_over: channel,
        to_outpost,
        from_outpost,
        counterparty_timeout: other.connection.counterparty_timeout,
        ibc_ics_20_sender: this
            .ibc
            .ok_or(ContractError::ICS20NotFound)?
            .channels
            .ok_or(ContractError::ICS20NotFound)?
            .ics20
            .ok_or(ContractError::ICS20NotFound)?
            .sender,
        on_remote_asset: to_asset,
    })
}

pub(crate) fn ics20_message_hook(
    _: auth::Sudo,
    deps: Deps,
    msg: XcMessageData,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let packet: XcPacket = msg.packet;
    ensure_anonymous(&packet.program)?;
    deps.api.debug(&format!(
        "cvm::outpost::ibc::ics20:: received assets {:?}, packet assets {:?}",
        &info.funds, &packet.assets
    ));

    let assets: Result<XcFunds, ContractError> = info
        .funds
        .into_iter()
        .map(|coin| {
            let asset = crate::assets::get_local_asset_by_reference(
                deps,
                AssetReference::Native { denom: coin.denom },
            )?;
            Ok((asset.asset_id, coin.amount.into()))
        })
        .collect();
    let call_origin = CallOrigin::Remote {
        user_origin: packet.user_origin,
    };
    let execute_program = ExecuteProgramMsg {
        salt: packet.salt,
        program: packet.program,
        assets: assets?.into(),
        tip: Some(info.sender.to_string()),
    };

    let msg = ExecuteMsg::ExecuteProgramPrivileged {
        call_origin,
        execute_program,
    };
    let msg = wasm_execute(env.contract.address, &msg, Default::default())?;
    Ok(Response::new().add_submessage(SubMsg::reply_always(msg, ReplyId::ExecProgram.into())))
}

fn ensure_anonymous(program: &XcProgram) -> Result<()> {
    use cvm_runtime::Instruction::*;
    for ix in &program.instructions {
        match ix {
            Transfer { .. } => {}
            Exchange { .. } => {}
            Spawn { program, .. } => ensure_anonymous(program)?,
            _ => Err(ContractError::AnonymousCallsCanDoOnlyLimitedSetOfActions)?,
        }
    }
    Ok(())
}
