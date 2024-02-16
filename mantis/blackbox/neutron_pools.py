# generated by datamodel-codegen:
#   filename:  neutron_pools.json

from __future__ import annotations

from typing import List, Optional

from pydantic import BaseModel, Field


class Config(BaseModel):
    migrateToAddress: Optional[str] = None
    whitelisted: Optional[bool] = None


class Prices(BaseModel):
    token1Address: str
    token1PriceUsd: float
    token2Address: str
    token2PriceUsd: float


class Asset(BaseModel):
    id: str
    address: str
    amount: str
    symbol: str


class AstroRewards(BaseModel):
    apr: float
    apy: int
    day: float


class ProtocolRewards(BaseModel):
    apr: int
    apy: int
    day: int


class TotalRewards(BaseModel):
    apr: float
    apy: float
    day: float


class TradingFees(BaseModel):
    apr: float
    apy: float
    day: float


class Reward(BaseModel):
    symbol: str
    amountPerDay: str
    amountPerSecond: str
    priceUsd: float
    precision: int
    amountPerDayUsd: str
    yield_: float = Field(..., alias="yield")
    isExternal: bool


class JsonItem(BaseModel):
    poolAddress: str
    lpAddress: str
    dayVolumeUsd: float
    poolLiquidityUsd: float
    poolLiquidity: int
    rewardTokenSymbol: Optional[str] = None
    config: Optional[Config] = None
    feeRate: List[str]
    poolType: str
    isBlocked: bool
    prices: Prices
    stakeable: bool
    assets: List[Asset]
    name: str
    isNew: bool
    isIlliquid: bool
    isDeregistered: bool
    sortingAssets: List[str]
    astroRewards: AstroRewards
    protocolRewards: ProtocolRewards
    totalRewards: TotalRewards
    tradingFees: TradingFees
    rewards: List[Reward]


class Model(BaseModel):
    json_: List[JsonItem] = Field(..., alias="json")
