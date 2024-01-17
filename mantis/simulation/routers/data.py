# for alignment on input and output of algorithm
from functools import cache
import pandas as pd
from enum import Enum
from typing import TypeVar, Generic
from pydantic import BaseModel, validator
from strictly_typed_pandas import DataSet

TAssetId = TypeVar("TAssetId")
TNetworkId = TypeVar("TNetworkId")
TAmount = TypeVar("TAmount")

class AssetTransfers(BaseModel, Generic[TAssetId, TAmount],):
    # positive whole numbers, set key
    in_asset_id: TAssetId
    out_asset_id: TAssetId
    
    # this is positive whole number too
    # if it is hard if None, please fail if it is None - for now will be always some
    usd_fee_transfer: TAmount | None = None
    
    # amount of token on chain were it is 
    amount_of_in_token : TAmount
    
    # amount of token on chain where token can go 
    amount_of_out_token : TAmount
    
    # fee per million to transfer
    fee_per_million: int
        
    # do not care
    metadata: str | None = None
    @validator("metadata", pre=True, always=True)
    def replace_nan_with_None(cls, v):
        return None if isinstance(v, float) else v       
    
# pool are bidirectional, so so in can be out and other way
class AssetPairsXyk(BaseModel, Generic[TAssetId, TAmount],):
    # set key
    pool_id: TAssetId
    in_asset_id: TAssetId
    out_asset_id: TAssetId
    
    # assumed that all all CFMM take fee from pair token in proportion 
    fee_of_in_per_million: int
    fee_of_out_per_million: int 
    # in reality fee can be flat or in other tokens, but not for now
    
    weight_of_a: int 
    weight_of_b: int 
    # if it is hard if None, please fail if it is None - for now will be always some
    pool_value_in_usd: TAmount  | None = None
    
    @validator("pool_value_in_usd", pre=True, always=True)
    def replace_nan_with_None(cls, v):
        return v if v == v else None
        
    # total amounts in reserves R
    in_token_amount: TAmount
    out_token_amount: TAmount
    
    metadata: str | None = None    
    @validator("metadata", pre=True, always=True)
    def replace_nan_with_None(cls, v):
        return None if isinstance(v, float) else v    
    
# this is what user asks for
class Input(BaseModel, Generic[TAssetId, TAmount],):
    # natural set key is ordered pair (in_token_id, out_token_id)
    in_token_id: TAssetId
    out_token_id: TAssetId
    # tendered amount DELTA
    in_amount: TAmount
    # expected received amount LAMBDA
    out_amount: TAmount
    # if max is True, user wants to spent all in to get at least out
    # if max is False, user wants to get exact out, but spent as small as possible in
    # please fail if bool is False for now
    max: bool

    
class SingleInputAssetCvmRoute(BaseModel):
    pass    


# transfer assets
class Spawn(BaseModel):
    # amount to take with transfer
    # None means all (DELTA)
    in_asset_amount: int | None = None
    out_asset_id: int
    next: SingleInputAssetCvmRoute

class Exchange(BaseModel):
    # none means all (DELTA)
    in_asset_amount: int | None = None
    pool_id : int
    next: SingleInputAssetCvmRoute

# always starts with Input amount and asset
class SingleInputAssetCvmRoute(BaseModel):
    next: list[Exchange | Spawn]

SingleInputAssetCvmRoute.update_forward_refs()

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

T = TypeVar("T")

class PydanticDataSet(BaseModel, DataSet[T]):
    pass


# global labelling of assets and exchanges
class AllData(BaseModel):
    # DataSet inherits from DataFrame
    # If key is in first set, it cannot be in second set, and other way around
    asset_transfers : list[AssetTransfers]
    asset_pairs_xyk : list[AssetPairsXyk]
    # if None, than solution must not contain any joins after forks
    # so A was split into B and C, and then B and C were moved to be D
    # D must "summed" from 2 amounts must be 2 separate routes branches
    fork_joins : list[str] | None = None
    
    @property
    @cache
    def all_tokens(self) -> list[TAssetId]:
        set = set()
        for x in self.asset_pairs_xyk:
            set.add(x.in_asset_id)
            set.add(x.out_asset_id)
        for x in self.asset_transfers:
            set.add(x.in_asset_id)
            set.add(x.out_asset_id)    
    def index_of_token(self, token: TAssetId) -> int:
        return self.all_tokens().index(token)
    @property
    def tokens_count(self) -> int :
        return len(self.all_tokens())


# helpers to setup tests data

def test_all_data() -> AllData:
    asset_transfers =  PydanticDataSet[AssetTransfers](pd.read_csv("asset_transfers.csv"))
    assets_pairs_xyk=  PydanticDataSet[AssetPairsXyk](pd.read_csv("assets_pairs_xyk.csv"))
    return AllData(assets_pairs_xyk, asset_transfers)

def new_data(pairs: list[AssetPairsXyk], transfers: list[AssetTransfers]) -> AllData:
    return AllData(
        asset_pairs_xyk = list[AssetPairsXyk](pairs),
        asset_transfers = list[AssetTransfers](transfers),
        fork_joins= None,
    )

def new_input(in_token_id, out_token_id, in_amount, out_amount) -> Input:
    return Input(in_token_id = in_token_id, out_token_id = out_token_id, in_amount = in_amount, out_amount = out_amount, max = True)     

def new_pair(pool_id, in_asset_id, out_asset_id, fee_of_in_per_million, fee_of_out_per_million, weight_of_a, weight_of_b, pool_value_in_usd, in_token_amount, out_token_amount, metadata = None) -> AssetPairsXyk:
    return AssetPairsXyk(pool_id = pool_id, in_asset_id = in_asset_id, out_asset_id = out_asset_id, fee_of_in_per_million = fee_of_in_per_million, fee_of_out_per_million = fee_of_out_per_million, weight_of_a = weight_of_a, weight_of_b = weight_of_b, pool_value_in_usd = pool_value_in_usd, in_token_amount = in_token_amount, out_token_amount = out_token_amount, metadata = metadata)
    
def new_transfer(in_asset_id, out_asset_id, usd_fee_transfer, amount_of_in_token, amount_of_out_token, fee_per_million, metadata = None) -> AssetTransfers:
    return AssetTransfers(in_asset_id = in_asset_id, out_asset_id = out_asset_id, usd_fee_transfer = usd_fee_transfer, amount_of_in_token = amount_of_in_token, amount_of_out_token = amount_of_out_token, fee_per_million = fee_per_million, metadata = metadata)


def read_dummy_data(TEST_DATA_DIR: str = "./") -> AllData:
    return AllData(
        asset_pairs_xyk=[AssetPairsXyk(**row) for _index, row in pd.read_csv(TEST_DATA_DIR / "assets_pairs_xyk.csv").iterrows()],
        asset_transfers=[AssetTransfers(**row) for _index, row in pd.read_csv(TEST_DATA_DIR / "assets_transfers.csv").iterrows()],
    )