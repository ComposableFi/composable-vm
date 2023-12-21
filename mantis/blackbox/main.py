from blackbox.cvm_runtime.response_to_get_config import GetConfigResponse
from blackbox.models import AllData, CosmosChains, NeutronPoolsResponse, OsmosisPoolsResponse
from blackbox.neutron_pools import Model as NeutronPoolsModel
from blackbox.settings import setting
from cosmpy.aerial.config import NetworkConfig 
from cosmpy.aerial.contract import LedgerClient, LedgerContract
from fastapi import FastAPI
import blackbox.cvm_runtime.query as cvm_query
import requests

app = FastAPI()

@app.get("/status")
async def status():
    return {"status": "ok"}

# gets all data from all sources
@app.get("/data/all") 
async def get_data_all()-> AllData:
    cfg = NetworkConfig(
    chain_id="centauri-1",
    url="grpc+"+ setting.composable_cosmos_grpc,
    fee_minimum_gas_price=1,
    fee_denomination="ppica",
    staking_denomination="ppica",
    )
    client = LedgerClient(cfg)
    cvm_contract = LedgerContract(
        path=None, client = client, address= setting.cvm_address
    )
        
    skip_api = CosmosChains.parse_raw(requests.get(setting.skip_money+ "v1/info/chains").content)      
    cvm_registry = GetConfigResponse.parse_obj(cvm_contract.query(cvm_query.QueryMsg5()))
    cvm_registry = None
    osmosis_pools = OsmosisPoolsResponse.parse_raw(requests.get(setting.osmosis_pools).content)
    astroport_pools = NeutronPoolsResponse.parse_raw(requests.get(setting.astroport_pools).content).result.data   
    result = AllData(osmosis_pools = osmosis_pools.pools, cvm_registry = cvm_registry, astroport_pools = astroport_pools, cosmos_chains=skip_api)
    return result