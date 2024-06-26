use crate::{error::ContractError, state};
use cosmwasm_std::{entry_point, wasm_execute, Coin, DepsMut, Env, Event, Response};

use ibc_apps_more::hook::{IBCLifecycleComplete, SudoMsg};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn sudo(deps: DepsMut, env: Env, msg: SudoMsg) -> crate::error::Result {
    deps.api.debug(&format!(
        "cvm::outpost::sudo {}",
        serde_json_wasm::to_string(&msg)?
    ));
    match msg {
        SudoMsg::IBCLifecycleComplete(IBCLifecycleComplete::IBCAck {
            channel,
            sequence,
            ack,
            success,
        }) => {
            if !success {
                handle_transport_failure(deps, env, channel, sequence, ack)
            } else {
                Ok(Response::new())
            }
        }
        SudoMsg::IBCLifecycleComplete(IBCLifecycleComplete::IBCTimeout { channel, sequence }) => {
            handle_transport_failure(deps, env, channel, sequence, "timeout".to_string())
        }
    }
}

/// return funds to executor and sets final error
fn handle_transport_failure(
    deps: DepsMut,
    _env: Env,
    channel: ibc_core_host_types::identifiers::ChannelId,
    sequence: u64,
    reason: String,
) -> Result<cosmwasm_std::Response, ContractError> {
    deps.api.debug(
        format!(
            "cvm::outpost::handle::transport_failure {} {} {}",
            &channel, sequence, &reason
        )
        .as_str(),
    );
    let msg = cvm_runtime::executor::ExecuteMsg::SetErr { reason };
    let bridge_msg =
        crate::state::tracking::get_execution_track(deps.storage, channel.as_str(), sequence)?;
    let executor =
        crate::state::executors::get_by_origin(deps.as_ref(), bridge_msg.executor_origin)?;
    let mut response = Response::new();

    let assets = bridge_msg
        .msg
        .assets
        .into_iter()
        .filter_map(|(asset, amount)| {
            if let Ok(asset) = state::assets::ASSETS.load(deps.storage, asset) {
                Some(Coin {
                    denom: asset.denom(),
                    amount: amount.into(),
                })
            } else {
                None
            }
        })
        .collect();

    response = response.add_message(wasm_execute(executor.address, &msg, assets)?);
    Ok(response.add_event(Event::new("cvm::outpost::handle::transport_failure")))
}
