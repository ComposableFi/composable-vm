from pydantic import BaseModel

class Config(BaseModel):
    osmosis_pools: str = "https://app.osmosis.zone/api/pools?page=1&limit=1000&min_liquidity=500000" 
    astroport_pools: str = None
    composable_cosmos_rpc : str = None
    neutron_rpc : str = None
    osmosis_rpc : str = None
