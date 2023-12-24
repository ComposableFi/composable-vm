use crate::{
    assets, auth,
    batch::BatchResponse,
    error::{ContractError, Result},
    events::make_event,
    executor, msg,
    prelude::*,
    state,
    state::network::{self, load_this},
};

use cosmwasm_std::{
    entry_point, wasm_execute, Addr, BankMsg, Coin, CosmosMsg, Deps, DepsMut, Env, MessageInfo,
    Response, StdError,
};
use cvm_route::asset::{AssetReference, AssetToNetwork};
use cw20::{Cw20Contract, Cw20ExecuteMsg};

use cvm_runtime::{
    outpost::{BridgeExecuteProgramMsg, ConfigSubMsg},
    CallOrigin, ExecutorOrigin, Funds,
};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: msg::ExecuteMsg) -> Result {
    use msg::ExecuteMsg;
    let sender = &info.sender;
    let canonical_sender = deps.api.addr_canonicalize(sender.as_str())?;
    deps.api.debug(&format!(
        "cvm::outpost::execute sender on chain {}, sender cross chain {}",
        sender,
        &serde_json_wasm::to_string(&canonical_sender)?
    ));
    match msg {
        ExecuteMsg::Config(msg) => {
            let auth = auth::Admin::authorise(deps.as_ref(), &info)?;
            handle_config_msg(auth, deps, msg, &env).map(Into::into)
        }

        msg::ExecuteMsg::ExecuteProgram(execute_program) => {
            handle_execute_program(deps, env, info, execute_program)
        }

        msg::ExecuteMsg::ExecuteProgramPrivileged {
            call_origin,
            execute_program,
        } => {
            let auth = auth::Contract::authorise(&env, &info)?;
            handle_execute_program_privilleged(auth, deps, env, call_origin, execute_program)
        }

        msg::ExecuteMsg::BridgeForward(msg) => {
            let auth =
                auth::Executor::authorise(deps.as_ref(), &info, msg.executor_origin.clone())?;

            if !msg.msg.assets.0.is_empty() {
                super::ibc::ics20::handle_bridge_forward(auth, deps, info, msg, env.block)
            } else {
                super::ibc::ics27::handle_bridge_forward_no_assets(auth, deps, info, msg, env.block)
            }
        }
        msg::ExecuteMsg::MessageHook(msg) => {
            deps.api
                .debug(&format!("cvm::outpost::execute::message_hook {:?}", msg));

            let auth = auth::WasmHook::authorise(deps.as_ref(), &env, &info, msg.from_network_id)?;

            super::ibc::ics20::ics20_message_hook(auth, deps.as_ref(), msg, env, info)
        }
        msg::ExecuteMsg::Shortcut(msg) => handle_shortcut(deps, env, info, msg),
    }
}

fn handle_config_msg(
    auth: auth::Admin,
    mut deps: DepsMut,
    msg: ConfigSubMsg,
    env: &Env,
) -> Result<BatchResponse> {
    deps.api.debug(serde_json_wasm::to_string(&msg)?.as_str());
    match msg {
        ConfigSubMsg::ForceNetworkToNetwork(msg) => {
            network::force_network_to_network(auth, deps, msg)
        }
        ConfigSubMsg::ForceAsset(msg) => assets::force_asset(auth, deps, msg),
        ConfigSubMsg::ForceExchange(msg) => crate::state::exchange::force_exchange(auth, deps, msg),
        ConfigSubMsg::ForceRemoveAsset { asset_id } => {
            assets::force_remove_asset(auth, deps, asset_id)
        }
        ConfigSubMsg::ForceAssetToNetworkMap(AssetToNetwork {
            this_asset,
            other_network,
            other_asset,
        }) => {
            assets::force_asset_to_network_map(auth, deps, this_asset, other_network, other_asset)
        }
        ConfigSubMsg::ForceNetwork(msg) => network::force_network(auth, deps, msg),
        ConfigSubMsg::ForceInstantiate { user_origin, salt } => {
            executor::force_instantiate(auth, env.contract.address.clone(), deps, user_origin, salt)
        }
        ConfigSubMsg::Force(msgs) => {
            let mut aggregated = BatchResponse::new();
            for msg in msgs {
                let response = handle_config_msg(auth, deps.branch(), msg, env)?;
                aggregated.merge(response);
            }
            Ok(aggregated)
        }
    }
}

fn handle_shortcut(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: msg::ShortcutSubMsg,
) -> Result {
    // seems it would be hard each time to form big json to smoke test transfer, so will store some
    // routes and generate program on chain for testing
    Err(ContractError::NotImplemented)
}

/// Transfers assets from user.
/// In case of bank assets, transfers directly,
/// In case of cw20 asset transfers using messages.
/// If assets are non, default 100% of bank transferred assets or delegated via CW20.
fn transfer_from_user(
    deps: &DepsMut,
    self_address: Addr,
    user: Addr,
    host_funds: Vec<Coin>,
    program_funds: Option<Funds<Displayed<u128>>>,
) -> Result<(Vec<CosmosMsg>, Funds<Displayed<u128>>)> {
    deps.api
        .debug(serde_json_wasm::to_string(&(&program_funds, &host_funds))?.as_str());

    if let Some(program_funds) = program_funds {
        let mut transfers = Vec::with_capacity(program_funds.0.len());
        for (asset_id, program_amount) in program_funds.0.iter() {
            match assets::get_asset_by_id(deps.as_ref(), *asset_id)?.local {
                cvm_route::asset::AssetReference::Native { denom } => {
                    let Coin {
                        amount: host_amount,
                        ..
                    } = host_funds
                        .iter()
                        .find(|host_coin| host_coin.denom == denom)
                        .ok_or(ContractError::ProgramFundsDenomMappingToHostNotFound(
                            *asset_id, denom,
                        ))?;
                    if *program_amount != u128::from(*host_amount) {
                        return Err(ContractError::ProgramAmountNotEqualToHostAmount)?;
                    }
                }
                cvm_route::asset::AssetReference::Cw20 { contract } => {
                    transfers.push(Cw20Contract(contract).call(Cw20ExecuteMsg::TransferFrom {
                        owner: user.to_string(),
                        recipient: self_address.to_string(),
                        amount: (*program_amount).into(),
                    })?)
                } // cvm_route::asset::AssetReference::Erc20 { .. } => {
                  //     Err(ContractError::RuntimeUnsupportedOnNetwork)?
                  // }
            }
        }
        Ok((transfers, program_funds))
    } else {
        let mut program_funds: Funds<Displayed<u128>> = <_>::default();
        for coin in host_funds {
            let asset = assets::get_local_asset_by_reference(
                deps.as_ref(),
                AssetReference::Native { denom: coin.denom },
            )?;

            program_funds.0.push((asset.asset_id, coin.amount.into()));
        }
        // we cannot do same trick with CW20 as need to know CW20 address (and it has to support
        // Allowance query).
        // so it is implement CW20 receiver interface like Michal did for wallet
        Ok((vec![], program_funds))
    }
}

/// Handles request to execute an [`CVMProgram`].
///
/// This is the entry point for executing a program from a user.  Handling
pub(crate) fn handle_execute_program(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    execute_program: msg::ExecuteProgramMsg,
) -> Result {
    let tip = execute_program
        .tip
        .unwrap_or(env.contract.address.to_string());
    let this = msg::Outpost::new(env.contract.address);
    let call_origin = CallOrigin::Local {
        user: info.sender.clone(),
    };
    let (transfers, assets) = transfer_from_user(
        &deps,
        this.address(),
        info.sender,
        info.funds,
        execute_program.assets,
    )?;
    let execute_program = BridgeExecuteProgramMsg {
        salt: execute_program.salt,
        program: execute_program.program,
        assets,
        tip: Some(tip),
    };
    let msg = msg::ExecuteMsg::ExecuteProgramPrivileged {
        call_origin,
        execute_program,
    };
    let msg = this.execute(msg)?;
    Ok(Response::default().add_messages(transfers).add_message(msg))
}

/// Handle a request to execute a [`CVMProgram`].
/// Only the gateway is allowed to dispatch such operation.
/// The gateway must ensure that the `CallOrigin` is valid as the router does not do further
/// checking on it.
pub(crate) fn handle_execute_program_privilleged(
    _: auth::Contract,
    deps: DepsMut,
    env: Env,
    call_origin: CallOrigin,
    msg::BridgeExecuteProgramMsg {
        salt,
        program,
        assets,
        tip,
    }: msg::BridgeExecuteProgramMsg,
) -> Result {
    let config = load_this(deps.storage)?;
    let executor_origin = ExecutorOrigin {
        user_origin: call_origin.user(config.network_id),
        salt: salt.clone(),
    };
    let executor = state::executors::get_by_origin(deps.as_ref(), executor_origin.clone()).ok();
    if let Some(state::executors::ExecutorItem { address, .. }) = executor {
        deps.api
            .debug("cvm::outpost::execute:: reusing existing executor and adding funds");
        let response = send_funds_to_executor(deps.as_ref(), address.clone(), assets)?;
        let wasm_msg = wasm_execute(
            address.clone(),
            &cvm_runtime::executor::ExecuteMsg::Execute {
                tip: tip
                    .map(|x| deps.api.addr_validate(&x))
                    .ok_or(ContractError::AccountInProgramIsNotMappableToThisChain)?
                    .unwrap_or(env.contract.address),
                program,
            },
            vec![],
        )?;
        Ok(response
            .add_event(make_event("route.execute").add_attribute("executor", address.into_string()))
            .add_message(wasm_msg))
    } else {
        // First, add a callback to instantiate an executor (which we later get the result
        // and save it)
        let executor_code_id = match config.outpost.ok_or(StdError::generic_err("outpost was was not set"))? {
            msg::OutpostId::CosmWasm {
                executor_code_id, ..
            } => executor_code_id,
            // msg::OutpostId::Evm { .. } => {
            //     Err(ContractError::BadlyConfiguredRouteBecauseThisChainCanSendOnlyFromCosmwasm)?
            // }
        };
        deps.api.debug("instantiating executor");
        let this = msg::Outpost::new(env.contract.address);

        let executor_instantiate_submsg = crate::executor::instantiate(
            deps.as_ref(),
            this.address(),
            executor_code_id,
            &executor_origin,
            salt,
        )?;

        // Secondly, call itself again with the same parameters, so that this functions goes
        // into `Ok` state and properly executes the executor
        let execute_program = cvm_runtime::outpost::BridgeExecuteProgramMsg {
            salt: executor_origin.salt,
            program,
            assets,
            tip,
        };
        let msg = msg::ExecuteMsg::ExecuteProgramPrivileged {
            call_origin,
            execute_program,
        };
        let self_call_message = this.execute(msg)?;

        Ok(Response::new()
            .add_event(make_event("route.create"))
            .add_submessage(executor_instantiate_submsg)
            .add_message(self_call_message))
    }
}

/// Transfer funds attached to a [`CVMProgram`] before dispatching the program to the executor.
fn send_funds_to_executor(
    deps: Deps,
    executor_address: Addr,
    funds: Funds<cvm_runtime::shared::Displayed<u128>>,
) -> Result {
    let mut response = Response::new();
    let executor_address = executor_address.into_string();
    for (asset_id, amount) in funds.0 {
        // Ignore zero amounts
        if amount == 0 {
            continue;
        }
        deps.api.debug("cvm::outpost:: sending funds");

        let msg = match assets::get_asset_by_id(deps, asset_id)?.local {
            cvm_route::asset::AssetReference::Native { denom } => BankMsg::Send {
                to_address: executor_address.clone(),
                amount: vec![Coin::new(amount.into(), denom)],
            }
            .into(),
            cvm_route::asset::AssetReference::Cw20 { contract } => {
                let contract = Cw20Contract(contract);
                contract.call(Cw20ExecuteMsg::Transfer {
                    recipient: executor_address.clone(),
                    amount: amount.into(),
                })?
            } //cvm_route::asset::AssetReference::Erc20 { .. } => Err(ContractError::RuntimeUnsupportedOnNetwork)?,
        };
        response = response.add_message(msg);
    }
    Ok(response)
}
