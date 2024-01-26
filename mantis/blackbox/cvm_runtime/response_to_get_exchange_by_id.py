# generated by datamodel-codegen:
#   filename:  response_to_get_exchange_by_id.json

from __future__ import annotations

from typing import Union

from pydantic import BaseModel, ConfigDict, Field, RootModel, conint


class Addr(RootModel[str]):
    root: str = Field(
        ...,
        description="A human readable address.\n\nIn Cosmos, this is typically bech32 encoded. But for multi-chain smart contracts no assumptions should be made other than being UTF-8 encoded and of reasonable length.\n\nThis type represents a validated address. It can be created in the following ways 1. Use `Addr::unchecked(input)` 2. Use `let checked: Addr = deps.api.addr_validate(input)?` 3. Use `let checked: Addr = deps.api.addr_humanize(canonical_addr)?` 4. Deserialize from JSON. This must only be done from JSON that was validated before such as a contract's state. `Addr` must not be used in messages sent by the user because this would result in unvalidated instances.\n\nThis type is immutable. If you really need to mutate it (Really? Are you sure?), create a mutable copy using `let mut mutable = Addr::to_string()` and operate on that `String` instance.",
    )


class DisplayedForUint128(RootModel[str]):
    root: str = Field(
        ...,
        description='A wrapper around a type which is serde-serialised as a string.\n\nFor serde-serialisation to be implemented for the type `T` must implement `Display` and `FromStr` traits.\n\n```rust use cvm::shared::Displayed;\n\n#[derive(serde::Serialize, serde::Deserialize)] struct Foo { value: Displayed<u64> }\n\nlet encoded = serde_json_wasm::to_string(&Foo { value: Displayed(42) }).unwrap(); assert_eq!(r#"{"value":"42"}"#, encoded);\n\nlet decoded = serde_json_wasm::from_str::<Foo>(r#"{"value":"42"}"#).unwrap(); assert_eq!(Displayed(42), decoded.value); ```',
    )


class OsmosisPoolManagerModuleV1Beta1(BaseModel):
    pool_id: conint(ge=0)
    token_a: str
    token_b: str


class ExchangeType5(BaseModel):
    model_config = ConfigDict(
        extra="forbid",
    )
    osmosis_pool_manager_module_v1_beta1: OsmosisPoolManagerModuleV1Beta1


class AstroportRouterContract(BaseModel):
    address: Addr
    token_a: str
    token_b: str


class ExchangeType6(BaseModel):
    model_config = ConfigDict(
        extra="forbid",
    )
    astroport_router_contract: AstroportRouterContract


class ExchangeType(RootModel[Union[ExchangeType5, ExchangeType6]]):
    root: Union[ExchangeType5, ExchangeType6]


class NetworkId(RootModel[conint(ge=0)]):
    root: conint(ge=0) = Field(
        ...,
        description="Newtype for CVM networks ID. Must be unique for each network and must never change. This ID is an opaque, arbitrary type from the CVM protocol and no assumption must be made on how it is computed.",
    )


class ExchangeItem(BaseModel):
    """
    allows to execute Exchange instruction
    """

    exchange: ExchangeType
    exchange_id: DisplayedForUint128
    network_id: NetworkId


class GetExchangeResponse(BaseModel):
    exchange: ExchangeItem
