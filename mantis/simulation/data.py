# for alignment on input and output of algorithm
import pandas as pd

from typing import TypeVar
from strictly_typed_pandas import DataSet

TAssetId = TypeVar("TAssetId")
TNetworkId = TypeVar("TNetworkId")

class AssetTransfers:
    # positive whole numbers, set key
    in_asset_id: int
    out_asset_id: int
    
    # this is positive whole number too
    # if it is hard if None, please fail if it is None - for now will be always some
    usd_fee_transfer: int | None
    
    # do not care
    metadata: str | None
    
class AssetPairsXyk:
    # set key
    in_asset_id: int
    out_asset_id: int
    
    fee_of_in_per_million: int
    fee_of_out_per_million: int 
    weight_of_a: int 
    weight_of_b: int 
    # if it is hard if None, please fail if it is None - for now will be always some
    pool_value_in_usd: int | None
    a_amount: int
    b_amount: int
    
    metadata: str | None
    
# this is what user asks for
class Input:
    in_token_id: int
    out_token_id: int
    in_amount: int
    out_amount: int
    # if max is True, user wants to spent all in to get at least out
    # if max is False, user wants to get exact out, but spent as small as possible in
    # please fail if bool is False for now
    max: bool

class AllData():
    # DataSet inherits from DataFrame
    # If key is in first set, it cannot be in second set, and other way around
    asset_transfers : DataSet[AssetTransfers]
    asset_pairs_xyk : DataSet[AssetPairsXyk]

def test_all_data() -> AllData:
    asset_transfers =  DataSet[AssetTransfers](pd.read_csv("asset_transfers.csv"))
    assets_pairs_xyk=  DataSet[AssetPairsXyk](pd.read_csv("assets_pairs_xyk.csv"))
    return AllData(assets_pairs_xyk, asset_transfers)