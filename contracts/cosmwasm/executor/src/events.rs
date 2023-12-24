use cosmwasm_std::{Addr, Event};
use cvm_runtime::{
    exchange::ExchangeId, executor::CvmExecutorInstantiated, shared, ExecutorOrigin, NetworkId,
    UserId,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename = "cvm.executor.exchange.succeeded")]
pub struct CvmExecutorExchangeSucceeded {
    pub exchange_id: ExchangeId,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename = "cvm.executor.exchange.started")]
pub struct CvmExecutorExchangeStarted {
    pub exchange_id: ExchangeId,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename = "cvm.executor.execution.started")]
pub struct CvmExecutorExecutionStarted {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename = "cvm.executor.transferred")]
pub struct CvmExecutorTransferred {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename = "cvm.executor.owner.added")]
pub struct CvmExecutorOwnerAdded {
    pub owner: Vec<Addr>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename = "cvm.executor.owner.removed")]
pub struct CvmExecutorOwnerRemoved {
    pub owner: Vec<Addr>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename = "cvm.executor.execution.failed")]
pub struct CvmExecutorExchangeFailed {
    pub reason: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename = "cvm.executor.instruction.spawned")]
pub struct CvmExecutorInstructionSpawned {
    pub origin_network_id: NetworkId,
    pub origin_user_id: UserId,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename = "cvm.executor.self.failed")]
pub struct CvmExecutorSelfFailed {
    pub reason: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename = "cvm.executor.crosschain.failed")]
pub struct CvmExecutorCrosschainFailed {
    pub reason: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename = "cvm.executor.instruction.call.initiated")]
pub struct CvmExecutorInstructionCallInitiated {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename = "cvm.executor.step.executed")]
pub struct CvmExecutorStepExecuted {
    #[serde(
        serialize_with = "hex::serialize",
        deserialize_with = "hex::deserialize"
    )]
    #[cfg_attr(feature = "json-schema", schemars(schema_with = "String::json_schema"))]
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub tag: Vec<u8>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename = "cvm.executor.instruction.spawning")]
pub struct CvmExecutorInstructionSpawning {
    #[cfg_attr(feature = "json-schema", schemars(schema_with = "String::json_schema"))]
    pub network_id: NetworkId,
}

impl CvmExecutorInstructionSpawning {
    pub fn new(network_id: NetworkId) -> Event {
        Event::new("cvm.executor.instruction.spawning")
            .add_attribute("network_id", network_id.to_string())
    }
}

/// used to generate schema, so that each events schema is available in one place
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub enum CvmExecutor {
    StepExecuted(CvmExecutorStepExecuted),
    SelfFailed(CvmExecutorSelfFailed),
    ExchangeStarted(CvmExecutorExchangeStarted),
    InstructionCallInitiated(CvmExecutorInstructionCallInitiated),
    InstructionSpawned(CvmExecutorInstructionSpawned),
    ExchangeFailed(CvmExecutorExchangeFailed),
    OwnerRemoved(CvmExecutorOwnerRemoved),
    OwnerAdded(CvmExecutorOwnerAdded),
    ExecutionStarted(CvmExecutorExecutionStarted),
    Transferred(CvmExecutorTransferred),
    Instantiated(CvmExecutorInstantiated),
    Exchanged(CvmExecutorExchangeSucceeded),
    CrosschainFailed(CvmExecutorCrosschainFailed),
    ExecutorInstructionSpawning(CvmExecutorInstructionSpawning),
}

// beneath is something to be generate by macro
// https://github.com/CosmWasm/cosmwasm/discussions/1871

impl CvmExecutorCrosschainFailed {
    pub fn new(reason: String) -> Event {
        Event::new("cvm.executor.crosschain.failed").add_attribute("reason", reason)
    }
}

impl CvmExecutorStepExecuted {
    pub fn new(tag: &[u8]) -> Event {
        let mut event = Event::new("cvm.executor.step.executed");
        if !tag.is_empty() {
            event = event.add_attribute("tag", hex::encode(tag));
        }
        event
    }
}

impl CvmExecutorSelfFailed {
    pub fn new(reason: String) -> Event {
        Event::new("cvm.executor.self.failed").add_attribute("reason", reason)
    }
}

impl CvmExecutorExchangeStarted {
    pub fn new(exchange_id: ExchangeId) -> Event {
        Event::new("cvm.executor.exchange.started")
            .add_attribute("exchange_id", exchange_id.to_string())
    }
}

impl CvmExecutorInstructionCallInitiated {
    pub fn new() -> Event {
        Event::new("cvm.executor.instruction.call.initiated")
    }
}

impl CvmExecutorInstructionSpawned {
    pub fn new(
        origin_network_id: NetworkId,
        origin_user_id: UserId,
        network_id: NetworkId,
    ) -> Event {
        Event::new("cvm.executor.instruction.spawned")
            .add_attribute(
                "origin_network_id",
                serde_json_wasm::to_string(&origin_network_id)
                    .expect("network id is controlled by us and it is always serde"),
            )
            .add_attribute(
                "origin_user_id",
                serde_json_wasm::to_string(&origin_user_id)
                    .expect("user id is controlled by us and it is always serde"),
            )
            .add_attribute(
                "network_id",
                serde_json_wasm::to_string(&network_id)
                    .expect("network id is controlled by us and it is always serde"),
            )
    }
}

impl CvmExecutorExchangeFailed {
    pub fn new(reason: String) -> Event {
        Event::new("cvm.executor.exchange.failed").add_attribute("reason", reason)
    }
}

impl CvmExecutorOwnerRemoved {
    pub fn new(owners: Vec<Addr>) -> Event {
        let mut e = Event::new("cvm.executor.owner.removed");
        for owner in owners {
            e = e.add_attribute("owner", owner.to_string())
        }
        e
    }
}

impl CvmExecutorOwnerAdded {
    pub fn new(owners: Vec<Addr>) -> Event {
        let mut e = Event::new("cvm.executor.owner.added");
        for owner in owners {
            e = e.add_attribute("owner", owner.to_string())
        }
        e
    }
}

impl CvmExecutorExecutionStarted {
    pub fn new() -> Event {
        Event::new("cvm.executor.execution.started")
    }
}

impl CvmExecutorTransferred {
    pub fn new() -> Event {
        Event::new("cvm.executor.transferred")
    }
}

impl CvmExecutorExchangeSucceeded {
    pub fn new(exchange_id: ExchangeId) -> Event {
        Event::new("cvm.executor.exchanged").add_attribute("exchange_id", exchange_id.to_string())
    }
}
