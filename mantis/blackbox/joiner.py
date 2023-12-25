# this is up to take production data in heterogenous shape, and force it into pandas dataframe(s)
from blackbox.models import AllData
import pandas as pd

from blackbox.cvm_runtime.execute import AssetId, AssetItem, AssetReference,AssetReference1, Native

from ..simulation.data import TAssetId, TNetworkId 

def merge_join(input: AllData) -> pd.DataFrame:
    raise NotImplementedError()



# converting raw datas to frames
def test_bases():
    asset = AssetItem(asset_id=AssetId("42"), local= AssetReference(AssetReference1(Native("osmo")))
     

# converting asset A to B with USD price of transfer
def test_asset_transfers_to_frame():
    raise NotImplementedError()
    
# converting pools with USD and native amounts, fees 
def test_pools_to_frame():
    raise NotImplementedError()

