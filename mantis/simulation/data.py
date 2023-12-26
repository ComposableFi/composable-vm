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

class AllData():
    # DataSet inherits from DataFrame
    # If key is in first set, it cannot be in second set, and other way around
    asset_transfers : DataSet[AssetTransfers]
    asset_pairs_xyk : DataSet[AssetPairsXyk]

def test_all_data() -> AllData:
    asset_transfers =  DataSet[AssetTransfers](pd.read_csv("asset_transfers.csv"))
    assets_pairs_xyk=  DataSet[AssetPairsXyk](pd.read_csv("assets_pairs_xyk.csv"))
    return AllData(assets_pairs_xyk, asset_transfers)