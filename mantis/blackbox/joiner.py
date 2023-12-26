# this is up to take production data in heterogenous shape, and force it into pandas dataframe(s)
from blackbox.models import AllData
import pandas as pd
from blackbox.cvm_runtime.execute import AssetId, AssetItem, AssetReference,AssetReference1, Native, NetworkId


def merge_join(input: AllData) -> pd.DataFrame:
    assets =  




# converting raw datas to frames
def test_bases():
    asset = AssetItem(asset_id=AssetId(__root__= "42"), local= AssetReference(__root__= AssetReference1(native= Native(denom="osmo"))), network_id= NetworkId(__root__= "2"))
    asset = AssetItem(asset_id=AssetId(__root__= "42"), local= AssetReference(__root__= AssetReference1(native= Native(denom="osmo"))), network_id= NetworkId(__root__= "2"))
    
    assert 3 == 2 

# converting asset A to B with USD price of transfer
def test_asset_transfers_to_frame():
    raise NotImplementedError()
    
    
# converting pools with USD and native amounts, fees 
def test_pools_to_frame():
    raise NotImplementedError()

