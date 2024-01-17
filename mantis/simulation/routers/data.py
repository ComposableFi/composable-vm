# for alignment on input and output of algorithm
from functools import cache
import pandas as pd
from enum import Enum
from typing import TypeVar, Generic
from pydantic import BaseModel, validator
from methodtools import lru_cache

# This is global unique ID for token(asset) or exchange(pool)
TId = TypeVar("TId")
TNetworkId = TypeVar("TNetworkId")
TAmount = TypeVar("TAmount")

class AssetTransfers(BaseModel, Generic[TId, TAmount],):
    # set key
    in_asset_id: TId
    out_asset_id: TId
    
    usd_fee_transfer: TAmount | None = None
    """_summary_
        this is positive whole number too
        if it is hard if None, please fail if it is None - for now will be always some
        In reality it can be any token
    """
    
    # amount of token on chain were it is 
    amount_of_in_token : TAmount
    
    # amount of token on chain wherane token can go 
    amount_of_out_token : TAmount
    
    # fee per million to transfer of asset itself
    fee_per_million: int | None = None
        
    # do not care
    metadata: str | None = None
    @validator("metadata", pre=True, always=True)
    def replace_nan_with_None(cls, v):
        return None if isinstance(v, float) else v       
    
# pool are bidirectional, so so in can be out and other way
class AssetPairsXyk(BaseModel, Generic[TId, TAmount],):
    # set key
    pool_id: TId
    in_asset_id: TId
    out_asset_id: TId
    
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
    def replace_nan_with_None(cls, v):
        return None if isinstance(v, float) else v    
    
# this is what user asks for
class Input(BaseModel, Generic[TId, TAmount],):
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



# global labelling of assets and exchanges
class AllData(BaseModel, Generic[TId, TAmount]):
    # DataSet inherits from DataFrame
    # If key is in first set, it cannot be in second set, and other way around
    asset_transfers : list[AssetTransfers[TId, TAmount]] = []
    asset_pairs_xyk : list[AssetPairsXyk[TId, TAmount]] = []
    # if None, than solution must not contain any joins after forks
    # so A was split into B and C, and then B and C were moved to be D
    # D must "summed" from 2 amounts must be 2 separate routes branches
    fork_joins : list[str] | None = None    
    usd_oracles: dict[TId, int] = [[]]
    """_summary_
      asset ids which we consider to be USD equivalents
      value - decimal exponent of token to make 1 USD
    """
        
    @property
    @cache
    def all_tokens(self) -> list[TId]:
        set = set()
        for x in self.asset_pairs_xyk:
            set.add(x.in_asset_id)
            set.add(x.out_asset_id)
        for x in self.asset_transfers:
            set.add(x.in_asset_id)
            set.add(x.out_asset_id)    
    def index_of_token(self, token: TId) -> int:
        return self.all_tokens().index(token)
    @property
    def tokens_count(self) -> int :
        return len(self.all_tokens())

    def token_price_in_usd(self, token: TId) -> float | None:
        """_summary_
        Either uses direct USD price from pool official oracle
        Or uses list of USD and tres to find pool for that assets directly with USD.     
        Returns:
            float | None: Value if found price, None if no price founds
        """
        hit = None
        for pair in self.asset_pairs_xyk:
            if pair.in_asset_id == token or pair.out_asset_id == token or pair.pool_value_in_usd:
                hit = pair
                break
        if hit:
            usd_volume = hit.pool_value_in_usd
            numerator = hit.weight_of_a if pair.in_asset_id == token else hit.weight_of_b
            denum = hit.weight_of_a + hit.weight_of_b
            top = numerator * usd_volume 
            btm = (hit.in_token_amount if pair.in_asset_id == token else hit.out_token_amount) * denum
            return top * 1.0 / btm
        else:
            # go over usd_oracles and than pools(breadth first search)
            return None
                
    

# helpers to setup tests data

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


class AssetPairsXyk1(BaseModel):
    # set key
    pool_id: int
    in_asset_id: int
    out_asset_id: int
    
    # assumed that all all CFMM take fee from pair token in proportion 
    fee_of_in_per_million: int
    fee_of_out_per_million: int 
    # in reality fee can be flat or in other tokens, but not for now
    
    weight_of_a: int 
    weight_of_b: int 
    # if it is hard if None, please fail if it is None - for now will be always some
    pool_value_in_usd: int  | None = None
    
    @validator("pool_value_in_usd", pre=True, always=True)
    def replace_nan_with_None(cls, v):
        return v if v == v else None
        
    # total amounts in reserves R
    in_token_amount: int
    out_token_amount: int
    
    metadata: str | None = None  

    
def read_dummy_data(TEST_DATA_DIR: str = "./") -> AllData:
    pairs = pd.read_csv(TEST_DATA_DIR / "assets_pairs_xyk.csv")
    return AllData(
        asset_pairs_xyk=[AssetPairsXyk(**row) for _index, row in pairs.iterrows()],
        asset_transfers=[AssetTransfers(**row) for _index, row in pd.read_csv(TEST_DATA_DIR / "assets_transfers.csv").iterrows()],
    )