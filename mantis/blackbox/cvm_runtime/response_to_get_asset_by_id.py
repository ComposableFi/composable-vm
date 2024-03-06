# generated by datamodel-codegen:
#   filename:  response_to_get_asset_by_id.json

from __future__ import annotations

from typing import Optional, Union

from pydantic import BaseModel, ConfigDict, Field, RootModel, conint


class Addr(RootModel[str]):
    root: str = Field(
        ...,
        description="A human readable address.\n\nIn Cosmos, this is typically bech32 encoded. But for multi-chain smart contracts no assumptions should be made other than being UTF-8 encoded and of reasonable length.\n\nThis type represents a validated address. It can be created in the following ways 1. Use `Addr::unchecked(input)` 2. Use `let checked: Addr = deps.api.addr_validate(input)?` 3. Use `let checked: Addr = deps.api.addr_humanize(canonical_addr)?` 4. Deserialize from JSON. This must only be done from JSON that was validated before such as a contract's state. `Addr` must not be used in messages sent by the user because this would result in unvalidated instances.\n\nThis type is immutable. If you really need to mutate it (Really? Are you sure?), create a mutable copy using `let mut mutable = Addr::to_string()` and operate on that `String` instance.",
    )


class AssetId(RootModel[str]):
    root: str = Field(
        ...,
        description='Newtype for CVM assets ID. Must be unique for each asset and must never change. This ID is an opaque, arbitrary type from the CVM protocol and no assumption must be made on how it is computed.',
    )


class Native(BaseModel):
    denom: str


class AssetReference5(BaseModel):
    """
    Definition of an asset native to some chain to operate on. For example for Cosmos CW and EVM chains both CW20 and ERC20 can be actual. So if asset is local or only remote to some chain depends on context of network or connection. this design leads to some dummy matches, but in general unifies code (so that if one have to solve other chain route it can)
    """

    model_config = ConfigDict(
        extra='forbid',
    )
    native: Native


class Cw20(BaseModel):
    contract: Addr


class AssetReference6(BaseModel):
    """
    Definition of an asset native to some chain to operate on. For example for Cosmos CW and EVM chains both CW20 and ERC20 can be actual. So if asset is local or only remote to some chain depends on context of network or connection. this design leads to some dummy matches, but in general unifies code (so that if one have to solve other chain route it can)
    """

    model_config = ConfigDict(
        extra='forbid',
    )
    cw20: Cw20


class AssetReference(RootModel[Union[AssetReference5, AssetReference6]]):
    root: Union[AssetReference5, AssetReference6] = Field(
        ...,
        description='Definition of an asset native to some chain to operate on. For example for Cosmos CW and EVM chains both CW20 and ERC20 can be actual. So if asset is local or only remote to some chain depends on context of network or connection. this design leads to some dummy matches, but in general unifies code (so that if one have to solve other chain route it can)',
    )


class NetworkId(RootModel[conint(ge=0)]):
    root: conint(ge=0) = Field(
        ...,
        description='Newtype for CVM networks ID. Must be unique for each network and must never change. This ID is an opaque, arbitrary type from the CVM protocol and no assumption must be made on how it is computed.',
    )


class PrefixedDenom(BaseModel):
    """
    A type that contains the base denomination for ICS20 and the source tracing information path.
    """

    base_denom: str = Field(
        ..., description='Base denomination of the relayed fungible token.'
    )
    trace_path: str = Field(
        ...,
        description='A series of `{port-id}/{channel-id}`s for tracing the source of the token.',
    )


class ForeignAssetId2(BaseModel):
    model_config = ConfigDict(
        extra='forbid',
    )
    ibc_ics20: PrefixedDenom


class ForeignAssetId(RootModel[ForeignAssetId2]):
    root: ForeignAssetId2


class BridgeAsset(BaseModel):
    location_on_network: ForeignAssetId


class AssetItem(BaseModel):
    asset_id: AssetId
    bridged: Optional[BridgeAsset] = Field(
        None,
        description='if asset was bridged, it would have way to identify bridge/source/channel',
    )
    local: AssetReference
    network_id: NetworkId = Field(
        ..., description='network id on which this asset id can be used locally'
    )


class GetAssetResponse(BaseModel):
    asset: AssetItem
