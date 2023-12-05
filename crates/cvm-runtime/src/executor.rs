use cosmwasm_std::Event;

use crate::prelude::*;
use crate::shared::XcProgram;
use crate::InterpreterOrigin;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[cfg_attr(
    all(feature = "json-schema", not(target_arch = "wasm32")),
    derive(schemars::JsonSchema)
)]
#[serde(rename_all = "snake_case")]
pub struct InstantiateMsg {
    /// Address of the gateway.
    pub gateway_address: String,
    /// The interpreter origin.
    pub interpreter_origin: InterpreterOrigin,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[cfg_attr(
    all(feature = "json-schema", not(target_arch = "wasm32")),
    derive(schemars::JsonSchema)
)]
#[serde(rename_all = "snake_case")]
pub struct Step {
    /// Tip party facilitated bridging and execution.
    pub tip: Addr,
    /// The current instruction pointer in the program.
    /// Note that the [`Step::program`] instructions are poped when executed, we can't rely on this
    /// instruction pointer to index into the instructions. In fact, this pointer tells us how many
    /// instructions we already consumed.
    pub instruction_pointer: u16,
    /// The next instructions to execute (actual program).
    pub program: XcProgram,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[cfg_attr(
    all(feature = "json-schema", not(target_arch = "wasm32")),
    derive(schemars::JsonSchema)
)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    /// Execute an CVM program
    Execute { tip: Addr, program: XcProgram },
    /// This is only meant to be used by the interpreter itself, otherwise it will return an error
    /// The existence of this message is to allow the execution of the `Call` instruction. Once we
    /// hit a call, the program queue the call and queue itself after it to ensure that the side
    /// effect of the call has been executed.
    ExecuteStep { step: Step },
    /// Add owners of this contract
    AddOwners { owners: Vec<Addr> },
    /// Remove owners from the contract
    RemoveOwners { owners: Vec<Addr> },
    /// spawn is cross chain, so sometimes errors are came from other blocks
    /// so gateway can set that error on interpreter
    SetErr { reason: String },
}

impl CvmInterpreterInstantiated {
    pub const NAME: &'static str = "cvm.executor.instantiated";
    pub const INTERPRETER_ORIGIN: &'static str = "interpreter_origin";
    #[cfg(feature = "cosmwasm")]
    pub fn new(interpreter_origin: &InterpreterOrigin) -> Event {
        use crate::shared::to_json_base64;

        Event::new(Self::NAME).add_attribute(
            Self::INTERPRETER_ORIGIN,
            to_json_base64(interpreter_origin).expect("origin is managed by"),
        )
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename = "cvm.executor.instantiated")]
pub struct CvmInterpreterInstantiated {
    pub interpreter_origin: String,
}
