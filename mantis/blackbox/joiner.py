# this is up to take production data in heterogenous shape, and force it into pandas dataframe(s)
from blackbox.models import AllData
import pandas as pd
from ..simulation.data import TAssetId, TNetworkId 
def merge_join(input: AllData) -> pd.DataFrame:
    raise NotImplementedError()





