use crate::{
    batch::BatchResponse,
    contract::ReplyId,
    error::{ContractError, Result},
    events::make_event,
    state::{self, network::load_this},
};
use cosmwasm_std::{
    to_json_binary, Deps, DepsMut, Reply, Response, StdError, StdResult, SubMsg, WasmMsg,
};

use cvm_runtime::{executor::CvmExecutorInstantiated, CallOrigin, ExecutorOrigin};

use crate::{auth, prelude::*};

pub(crate) fn force_instantiate(
    _: auth::Admin,
    outpost: Addr,
    deps: DepsMut,
    user_origin: Addr,
    salt: String,
) -> Result<BatchResponse> {
    let config = load_this(deps.storage)?;
    let executor_code_id = match config.outpost.expect("expected setup") {
        OutpostId::CosmWasm {
            executor_code_id: executor_code_id,
            ..
        } => executor_code_id,
        //OutpostId::Evm { .. } => Err(ContractError::RuntimeUnsupportedOnNetwork)?,
    };
    let salt = salt.into_bytes();

    let call_origin = CallOrigin::Local { user: user_origin };
    let executor_origin = ExecutorOrigin {
        user_origin: call_origin.user(config.network_id),
        salt: salt.clone(),
    };
    let msg = instantiate(
        deps.as_ref(),
        outpost,
        executor_code_id,
        &executor_origin,
        salt,
    )?;
    Ok(BatchResponse::new().add_submessage(msg).add_event(
        make_event("executor.forced")
            .add_attribute("executor_origin", executor_origin.to_string()),
    ))
}

pub fn instantiate(
    deps: Deps,
    admin: Addr,
    executor_code_id: u64,
    executor_origin: &ExecutorOrigin,
    salt: Vec<u8>,
) -> Result<SubMsg, ContractError> {
    let next_executor_id: u128 = state::executors::EXECUTORS_COUNT
        .load(deps.storage)
        .unwrap_or_default()
        + 1;

    let instantiate_msg = WasmMsg::Instantiate2 {
        admin: Some(admin.clone().into_string()),
        code_id: executor_code_id,
        msg: to_json_binary(&cvm_runtime::executor::InstantiateMsg {
            outpost_address: admin.into_string(),
            executor_origin: executor_origin.clone(),
        })?,
        funds: vec![],
        // and label has some unknown limits  (including usage of special characters)
        label: format!("cvm_executor_{}", &next_executor_id),
        // salt limit is 64 characters
        salt: to_json_binary(&salt)?,
    };
    let executor_instantiate_submessage =
        SubMsg::reply_on_success(instantiate_msg, ReplyId::InstantiateExecutor.into());
    Ok(executor_instantiate_submessage)
}

pub(crate) fn handle_instantiate_reply(deps: DepsMut, msg: Reply) -> StdResult<Response> {
    deps.api.debug(&format!(
        "cvm:: {}",
        serde_json_wasm::to_string(&msg).map_err(|e| StdError::generic_err(e.to_string()))?
    ));
    let response = msg.result.into_result().map_err(StdError::generic_err)?;
    // Catch the default `instantiate` event which contains `_contract_address` attribute that
    // has the instantiated contract's address
    let address = &response
        .events
        .iter()
        .find(|event| event.ty == "instantiate")
        .ok_or_else(|| StdError::not_found("instantiate event not found"))?
        .attributes
        .iter()
        .find(|attr| &attr.key == "_contract_address")
        .ok_or_else(|| StdError::not_found("_contract_address attribute not found"))?
        .value;
    let executor_address = deps.api.addr_validate(address)?;

    // Interpreter provides `network_id, user_id` pair as an event for the router to know which
    // pair is instantiated

    let event_name = format!("wasm-{}", CvmExecutorInstantiated::NAME);
    let executor_origin = &response
        .events
        .iter()
        .find(|event| event.ty.starts_with(&event_name))
        .ok_or_else(|| StdError::not_found("executor event not found"))?
        .attributes
        .iter()
        .find(|attr| attr.key == CvmExecutorInstantiated::EXECUTOR_ORIGIN)
        .ok_or_else(|| StdError::not_found("no data is returned from 'cvm_executor'"))?
        .value;
    let executor_origin =
        cvm_runtime::shared::decode_base64::<_, ExecutorOrigin>(executor_origin.as_str())?;

    let executor_id: u128 = state::executors::EXECUTORS_COUNT
        .load(deps.storage)
        .unwrap_or_default()
        + 1;
    let executor = state::executors::ExecutorItem {
        address: executor_address,
        executor_id: executor_id.into(),
    };

    state::executors::EXECUTORS_COUNT.save(deps.storage, &executor_id)?;
    state::executors::EXECUTORS.save(deps.storage, executor_id, &executor)?;
    state::executors::EXECUTOR_ORIGIN_TO_ID.save(
        deps.storage,
        executor_origin,
        &executor_id,
    )?;

    deps.api.debug("cvm:: saved executor");

    Ok(Response::new().add_event(
        make_event("cvm.executor.instantiated")
            .add_attribute("executor_id", executor_id.to_string()),
    ))
}
