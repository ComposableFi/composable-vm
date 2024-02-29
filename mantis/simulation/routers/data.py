"""
Input and output data to any algorithm for routing.
Connect semantic data model numpy indexed values and back.
"""
from __future__ import annotations

from enum import Enum
from fractions import Fraction
from typing import Generic, TypeVar, Union

import numpy as np
import pandas as pd
from disjoint_set import DisjointSet
from loguru import logger
from pydantic import BaseModel, model_validator, validator

from simulation.routers.oracles import merge_by_connection_from_existing

# This is global unique ID for token(asset) or exchange(pool)
TId = TypeVar("TId", int, str)
TNetworkId = TypeVar("TNetworkId", int, str)
TAmount = TypeVar("TAmount", int, str)

MINIMAL_TRANSACTION_USD_COST_DEFAULT: float = 0.00001
"""_summary_
Each venue eats some USD for transaction.
So we just cut noise.
"""


class Ctx(BaseModel, Generic[TAmount]):
    debug: bool = True
    """_summary_
     If set to true, solver must output maxima information about solution
    """
    integer: bool = True
    """_summary_
     If set too true, solver must solve only integer problems.
     All inputs to solver are really integers.
    """

    max_trade: float = 0.1
    """
    Do not trade if you will skew market
    """

    max_reserve_decimals: int = 4
    """_summary_
        If algorithm can not handle big numbers, it can be reduced to power of it
    """

    min_input_to_reserve_ratio: float = 0.0001
    """
    We consider that cannot get more than 1/this arbitrage
    """

    min_usd_reserve: float = 1

    min_input_in_usd: float = 0.0001
    minimal_trading_probability: float = 0.000001
    minimal_amount: float = 0.00001
    """_summary_
    Numerically minimal amount of change goes via venue is accepted, minimal trade.
    This is numeric amount, not value amount (oracalized amount) limit.
    Must be equal or larger than solver tolerance.
    """

    minimal_tradeable_number: float = 0.00000001
    """
    Not a value, but just physical number to eliminate from trade
    """

    mi_for_venue_count: int = 0
    """
    If venue count is small, can try MI solution because MI are slow in general
    """

    min_usd_venue_amount: float = 0.0000000001

    maximal_split_count: int = 10_0000
    """_summary_
    Do not split trade more than to these parts.
    Up to possible oracle error (do not set it less than expected oracle mistake multiplier)
    """

    input_consumed_ratio : float = 0.9

    max_hops_usd: float = 0
    """
    total amount to pay for transfers fees
    reduces desire of router to spawn
    we just measure it as loss of funds
    """

    depth_of_route: int = 3
    """_summary_
    Avoid too deep routes.
    """

    max_venues_usd: int = 5
    """
    Prevents arbitrage but allows for simpler routes if set small.
    """

    @property
    def max_reserve(self):
        return 10**self.max_reserve_decimals


class TwoTokenConverter:
    in_asset_id: TId
    out_asset_id: TId

    def zero(self):
        self.in_token_amount = 0
        self.out_token_amount = 0


class AssetTransfers(
    BaseModel,
    TwoTokenConverter,
    Generic[TId, TAmount],
):
    venue_fixed_costs_in_usd: float
    """_summary_
        this is positive whole number too
        if it is hard if None, please fail if it is None - for now will be always some
        In reality it can be any token.
        Fixed costs $q_i$ >= 0s
    """

    in_token_amount: TAmount
    """
     Tendered amount of token on chain were it is.
     Must be like escrowed amount.
    """

    out_token_amount: TAmount
    """
      Expected received amount LAMBDA.
      Must be like liquid amount of this token minted.
    """

    # fee per million to transfer of asset itself
    fee_per_million: int

    # do not care
    metadata: str | None = None

    @validator("metadata", pre=True, always=True)
    def replace_nan_with_None(cls, v):
        return None if isinstance(v, float) else v


class AssetPairsXyk(
    BaseModel,
    Generic[TId, TAmount],
):
    """_summary_
    Strictly 2 asset pool with weights (1/1 for original uniswap).
    Pool are bidirectional, so so in can be out and other way
    """

    # set key
    pool_id: TId
    in_asset_id: TId
    out_asset_id: TId

    # assumed that all all CFMM take fee from pair token in proportion
    fee_of_in_per_million: int
    fee_of_out_per_million: int
    # in reality fee can be flat or in other tokens, but not for now

    weight_a: int
    weight_b: int
    # if it is hard if None, please fail if it is None - for now will be always some
    pool_value_in_usd: TAmount | None = None
    venue_fixed_costs_in_usd: float = 0.00001

    @validator("pool_value_in_usd", pre=True, always=True)
    def replace_nan_with_None(cls, v):
        return v if v == v else None

    # total amounts in reserves R
    in_token_amount: TAmount
    out_token_amount: TAmount

    @property
    def value_of_a_in_usd(self):
        """
        How much 1 of a costs in USD
        """
        return self.a_usd / self.in_token_amount

    @property
    def value_of_b_in_usd(self):
        """
        How much 1 of a costs in USD
        """
        return self.b_usd / self.out_token_amount

    @property
    def a_usd(self):
        return self.pool_value_in_usd * self.weight_b / (self.weight_a + self.weight_b)

    @property
    def b_usd(self):
        return self.pool_value_in_usd * self.weight_a / (self.weight_a + self.weight_b)

    @property
    def weighted_a(self):
        return self.in_token_amount**self.weight_a

    @property
    def weighted_b(self):
        return self.out_token_amount**self.weight_b

    @property
    def weighted_volume(self):
        return self.weighted_a * self.weighted_b

    @property
    def fee_in(self) -> float:
        """_summary_
        Part of amount taken as fee
        """
        self.fee_of_in_per_million / 1_000_000

    @property
    def fee_out(self) -> float:
        """_summary_
        Part of amount taken as fee
        """
        self.fee_of_out_per_million / 1_000_000

    metadata: str | None = None

    @validator("metadata", pre=True, always=True)
    def replace_nan_with_metadata_None(cls, v):
        return None if isinstance(v, float) else v


# this is what user asks for
class Input(
    BaseModel,
    Generic[TId, TAmount],
):
    # natural set key is ordered pair (in_token_id, out_token_id)
    in_token_id: TId
    out_token_id: TId
    # tendered amount DELTA
    in_amount: TAmount
    # expected received amount LAMBDA
    out_amount: TAmount
    # if max is True, user wants to spent all in to get at least out
    # if max is False, user wants to get exact out, but spent as small as possible in
    # please fail if bool is False for now
    max: bool


# @dataclass
class Spawn(BaseModel, Generic[TId, TAmount]):
    """
    cross chain transfer assets
    """

    in_asset_id: TId | None = None

    in_asset_amount: TAmount | None = None
    """
    amount to take with transfer
    (delta)
    """
    out_asset_amount: TAmount | None = None

    out_asset_id: TId | None = None
    next: list[Union[Exchange, Spawn]]


# @dataclass
class Exchange(BaseModel, Generic[TId, TAmount]):
    in_asset_amount: TAmount
    """
    none means all (DELTA)
    """

    out_asset_amount: TAmount
    """_summary_
    Means expected minimal amount.
    """

    in_asset_id: TId

    out_asset_id: TId

    pool_id: TId
    next: list[Union[Exchange, Spawn]]

    @model_validator(mode="after")
    def after(self) -> "Exchange":
        # assert self.out_asset_amount > 0
        # assert self.in_asset_amount > 0
        assert self.in_asset_id != self.out_asset_id
        return self


class SingleInputAssetCvmRoute(BaseModel):
    """
    always starts with Input asset_id
    """

    in_amount: int
    next: list[Union[Exchange, Spawn]]


SingleInputAssetCvmRoute.model_rebuild()


class SolutionType(Enum):
    # really to find any solution
    FAILED = 0
    # all assets will be solved with limit
    FULL = 1
    # will be solved under desired limit
    UNDER_LIMIT = 2
    # will solve within limits, bat only part of assets
    PARTIAL = 3


class Output(BaseModel):
    # str describing failure to find any solution
    route: SingleInputAssetCvmRoute | str
    solution_type: SolutionType


class AllData(BaseModel, Generic[TId, TAmount]):
    """
    Immutable(frozen) after creation (so can cache everything)
    Global labelling of assets and exchanges
    It can be expected that original value of input can be up 2^128 integer (so it is edge case).
    Expected real case 2^64.
    """

    #   If key is in first set, it cannot be in second set, and other way around
    asset_transfers: list[
        AssetTransfers[TId, TAmount]
    ] | None = None  #    If we want to set default values, we need to save data structure in default
    asset_pairs_xyk: list[
        AssetPairsXyk[TId, TAmount]
    ] | None = None  #    If we want to set default values, we need to save data structure in default
    usd_oracles: dict[TId, float | None] | None = None
    """_summary_
      asset ids which we consider to be USD equivalents
      value - decimal exponent of token to make 1 USD
    """

    @property
    # @cache
    def all_tokens(self) -> list[TId]:
        tokens = []
        for x in self.asset_pairs_xyk:
            tokens.append(x.in_asset_id)
            tokens.append(x.out_asset_id)
        for x in self.asset_transfers:
            tokens.append(x.in_asset_id)
            tokens.append(x.out_asset_id)
        return list(set(tokens))

    def global_reservers_of(self, asset_id: TId) -> TAmount:
        """_summary_
        Goes over transfers path and finds all pools with same token which server estimate of total issuance
        """
        ds = DisjointSet()

        for pair in self.asset_pairs_xyk:
            ds.union(pair.in_asset_id, pair.in_asset_id)
            ds.union(pair.out_asset_id, pair.out_asset_id)

        for transfer in self.asset_transfers:
            ds.union(transfer.out_asset_id, transfer.in_asset_id)
        total_issuance = 0
        for span in ds.itersets():
            if asset_id in span:
                for asset_id in span:
                    total_issuance += self.total_reserves_of(asset_id)
                break
        return total_issuance

    def index_of_token(self, token: TId) -> int:
        return self.all_tokens.index(token)

    def assets_for_venue(self, venue: int) -> list[TId]:
        venue = self.venues_tokens[venue]
        return [venue[0], venue[1]]

    def get_index_in_all(self, venue: AssetPairsXyk | AssetTransfers) -> int:
        if isinstance(venue, AssetPairsXyk):
            return self.asset_pairs_xyk.index(venue)
        else:
            return len(self.asset_pairs_xyk) + self.asset_transfers.index(venue)

    @property
    def venue_fixed_costs_in_usd(self) -> list[float]:
        """_summary_
        fixed costs of using venue in USD
        """
        costs = []
        for x in self.asset_pairs_xyk:
            costs.append(x.venue_fixed_costs_in_usd)
        for x in self.asset_transfers:
            costs.append(x.venue_fixed_costs_in_usd)
        return costs

    def venue_fixed_costs_in(self, token: TId) -> list[float]:
        """_summary_
        Converts fixed price of all venues to token
        """
        token_price_in_usd = self.token_price_in_usd(token)
        if not token_price_in_usd:
            raise Exception(
                "WARN: mantis::simulation::routers:: token has no price found to compare to fixed costs, so fixed costs would be considered 1 or 0"
            )

        in_received_token = []
        for fixed_cost_in_usd in self.venue_fixed_costs_in_usd:
            if not token_price_in_usd and fixed_cost_in_usd > 0:
                in_received_token.append(0.000001)
            elif not token_price_in_usd and fixed_cost_in_usd == 0:
                in_received_token.append(0)
            else:
                in_received_token.append(fixed_cost_in_usd / token_price_in_usd)
        return in_received_token

    @property
    def venues_proportional_reductions(self) -> list[Fraction]:
        """_summary_
        remaining_% = 1 - fee_%
        """
        denom = 1_000_000
        reduced = []
        for x in self.asset_pairs_xyk:
            fee = Fraction(max(x.fee_of_in_per_million, x.fee_of_in_per_million), denom)
            reduced.append(1 - fee)
        for x in self.asset_transfers:
            fee = Fraction(max(x.fee_per_million, x.fee_per_million), denom)
            reduced.append(1 - fee)
        return reduced

    @property
    def all_reserves(self) -> list[np.ndarray[int]]:
        """_summary_
        Produces reserves per asset in next order
        - Balancer/UniV2 pools
        - Cross chain transfers (escrow amounts and liquidity)
        """
        reserves = []
        for x in self.asset_pairs_xyk:
            reserves.append(np.array([x.in_token_amount, x.out_token_amount]))
        for x in self.asset_transfers:
            reserves.append(
                np.array(
                    [
                        self.maximal_reserves_of(x.in_asset_id),
                        self.maximal_reserves_of(x.out_asset_id),
                    ]
                )
            )
        return reserves

    def venue_by_index(self, index) -> Union[AssetTransfers, AssetPairsXyk]:
        if index < len(self.asset_pairs_xyk):
            return self.asset_pairs_xyk[index]
        return self.asset_transfers[index - len(self.asset_pairs_xyk)]

    @property
    def transfers_disjoint_set(self) -> DisjointSet:
        """
        Find all assets which can be routed one into other
        """
        routable = DisjointSet()
        for transfer in self.asset_transfers:
            routable.union(transfer.in_asset_id, transfer.out_asset_id)
        return routable

    def transfer_to_exchange(self, from_asset_id: TId) -> list[TId]:
        """
        if can transfer asset to reach exchanges.
        return list of exchanges
        """
        result = set()
        routable = self.transfers_disjoint_set
        for exchange in self.asset_pairs_xyk:
            if routable.connected(from_asset_id, exchange.in_asset_id) or routable.connected(
                from_asset_id, exchange.out_asset_id
            ):
                result.add(exchange.pool_id)
        return list(result)

    def maximal_reserves_of(self, asset_id: TId) -> TAmount:
        """_summary_
        Given token find maximal reserve venue it across all venues.
        If case it able to find escrow and liquidity for transfer venue it uses that value.
        If not, it uses maximal pool for the token.
        In case of both fails, it sets max numeric value for that.
        """
        value = 0
        for venue in self.asset_pairs_xyk:
            if venue.in_asset_id == asset_id:
                value = max(value, value, venue.in_token_amount)
            if venue.out_asset_id == asset_id:
                value = max(value, venue.out_token_amount)
        if value > 0:
            return value

        exchanges = self.transfer_to_exchange(asset_id)
        for exchange_id in exchanges:
            for venue in self.asset_pairs_xyk:
                if venue.pool_id == exchange_id:
                    if self.transfers_disjoint_set.connected(venue.in_asset_id, asset_id):
                        value = max(value, venue.in_token_amount)
                    elif self.transfers_disjoint_set.connected(venue.out_asset_id, asset_id):
                        value = max(value, venue.out_token_amount)
                    else:
                        raise Exception(f"asset {asset_id} not connected to {venue}")
        if value > 0:
            return value

        for venue in self.asset_transfers:
            if venue.in_asset_id == asset_id:
                value = max(value, venue.in_token_amount)
            if venue.out_asset_id == asset_id:
                value = max(value, venue.out_token_amount)
        if value > 0:
            return value

        if value == 0:
            logger.trace(f"asset reservers not found {asset_id}")
        return 0

    def total_reserves_of(self, token: TId) -> int:
        """
        Approximation of global reserves of token in all venues
        """
        global_value_locked = 0
        for x in self.asset_pairs_xyk:
            if x.out_asset_id == token:
                global_value_locked += x.out_token_amount
            if x.in_asset_id == token:
                global_value_locked += x.in_token_amount
        if global_value_locked > 0:
            return global_value_locked
        return global_value_locked

    @property
    # @lru_cache
    def venues_tokens(self) -> list[list[TId]]:
        """
        Tokens in venues.
        """
        venues = []
        for x in self.asset_pairs_xyk:
            venues.append([x.in_asset_id, x.out_asset_id])
        for x in self.asset_transfers:
            venues.append([x.in_asset_id, x.out_asset_id])
        return venues

    @property
    def venues(self) -> list[Union[AssetPairsXyk, AssetTransfers]]:
        all = []
        all.extend(self.asset_pairs_xyk)
        all.extend(self.asset_transfers)
        return all

    def venue(self, i: int):
        reserves = self.all_reserves
        venues = self.venues_tokens
        return (venues[i], reserves[i])

    @property
    # @lru_cache
    def tokens_count(self) -> int:
        """_summary_
        in solver global matrices NxN
        """
        return len(self.all_tokens)

    @property
    def venues_count(self) -> int:
        """_summary_
        Number of ways any one specific token can be converted to other one.
        In solver local matrix row count
        """
        return len(self.asset_pairs_xyk) + len(self.asset_transfers)

    def merge_oracles(self):
        """_summary_
        Given original USD oracle and pools, merge them to one dictionary
        """
        oracles = {}
        if self.usd_oracles and any(self.usd_oracles):
            for key, value in self.usd_oracles.items():
                if value and value > 0:
                    oracles[key] = value
                             
        transfers = [(x.in_asset_id, x.out_asset_id) for x in self.asset_transfers]
        oracles = merge_by_connection_from_existing(oracles, transfers)
        # merge with pool oracles
        
        transfers = self.transfers_disjoint_set
        for other_asset_id in self.all_tokens:
            if other_asset_id not in oracles or oracles[other_asset_id] is None or oracles[other_asset_id] == 0:
                for pair in self.asset_pairs_xyk:
                    if pair.pool_value_in_usd:
                        if transfers.connected(pair.in_asset_id, other_asset_id):
                            assert pair.value_of_a_in_usd > 0
                            oracles[other_asset_id] = pair.value_of_a_in_usd
                            break
                        if transfers.connected(pair.out_asset_id, other_asset_id):
                            assert pair.value_of_b_in_usd > 0
                            oracles[other_asset_id] = pair.value_of_b_in_usd
                            break
        self.usd_oracles = oracles
        

    # @property
    # @lru_cache
    def token_price_in_usd(self, asset_id: TId) -> float | None:
        """_summary_
        How much 1 amount of token is worth in USD
        """

        if self.usd_oracles.get(asset_id, None):
            return self.usd_oracles[asset_id]
        else:
            logger.error(f"oracle not found for token {asset_id}, which means there is no connection of token to USD")
            return 0

    @model_validator(mode="after")
    def after(self):
        assert len(self.asset_pairs_xyk) + len(self.asset_transfers) == self.venues_count


# helpers to setup tests data


def new_data(pairs: list[AssetPairsXyk], transfers: list[AssetTransfers], usd_oracles=None) -> AllData:
    return AllData(
        asset_pairs_xyk=list[AssetPairsXyk](pairs),
        asset_transfers=list[AssetTransfers](transfers),
        usd_oracles=usd_oracles,
    )


def new_input(in_token_id, out_token_id, in_amount, out_amount) -> Input:
    return Input(
        in_token_id=in_token_id,
        out_token_id=out_token_id,
        in_amount=in_amount,
        out_amount=out_amount,
        max=True,
    )


def new_pair(
    pool_id,
    in_asset_id,
    out_asset_id,
    fee_of_in_per_million,
    fee_of_out_per_million,
    weight_a,
    weight_b,
    pool_value_in_usd,
    in_token_amount,
    out_token_amount,
    venue_fixed_costs_in_usd: float = 0.00001,
    metadata=None,
) -> AssetPairsXyk:
    return AssetPairsXyk(
        pool_id=pool_id,
        in_asset_id=in_asset_id,
        out_asset_id=out_asset_id,
        fee_of_in_per_million=fee_of_in_per_million,
        fee_of_out_per_million=fee_of_out_per_million,
        weight_a=weight_a,
        weight_b=weight_b,
        pool_value_in_usd=pool_value_in_usd,
        in_token_amount=in_token_amount,
        out_token_amount=out_token_amount,
        venue_fixed_costs_in_usd=venue_fixed_costs_in_usd,
        metadata=metadata,
    )


def new_transfer(
    in_asset_id,
    out_asset_id,
    venue_fixed_costs_in_usd,
    in_token_amount,
    out_token_amount,
    fee_per_million,
    metadata=None,
) -> AssetTransfers:
    return AssetTransfers(
        in_asset_id=in_asset_id,
        out_asset_id=out_asset_id,
        venue_fixed_costs_in_usd=venue_fixed_costs_in_usd,
        in_token_amount=in_token_amount,
        out_token_amount=out_token_amount,
        fee_per_million=fee_per_million,
        metadata=metadata,
    )


def read_dummy_data(TEST_DATA_DIR: str = "./") -> AllData:
    pairs = pd.read_csv(TEST_DATA_DIR + "assets_pairs_xyk.csv")
    return AllData(
        asset_pairs_xyk=[AssetPairsXyk(**row) for _index, row in pairs.iterrows()],
        asset_transfers=[
            AssetTransfers(**row) for _index, row in pd.read_csv(TEST_DATA_DIR + "assets_transfers.csv").iterrows()
        ],
    )


def simulate_verify(all_data: AllData, input: Input, route: SingleInputAssetCvmRoute):
    """_summary_
    Given route and original data,
     traverse route doing trading.
    Verify route does not violates user limit neither promise from route
    """
    pass
