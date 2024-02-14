from typing import List
from blackbox.models import (
    AllData,
    OsmosisPoolsResponse,
)
from blackbox.settings import settings
from cosmpy.aerial.config import NetworkConfig
from cosmpy.aerial.contract import LedgerClient
from fastapi import FastAPI
import requests
from simulation.routers.angeris_cvxpy import cvxpy_to_data
from  simulation.routers import generic_linear
import uvicorn
from simulation.routers import test_generic_linear
from simulation.routers import data
import sys
import os
from pydantic import BaseModel
from simulation.routers.data import (
    AssetPairsXyk,
    Input,
    new_pair,
    read_dummy_data,
    AllData as CvmAllData,
)
from simulation.routers.data import Ctx, read_dummy_data, AllData as CvmAllData

app = FastAPI()


# 1. return csv data + data schema in 127.0.0.1:8000/docs
# https://github.com/ComposableFi/composable/issues/4434
@app.get("/data/dummy/everything")
async def data_dummy_everything() -> CvmAllData:
    data = read_dummy_data("./simulation/routers/data/")
    return data


@app.get("/data/dummy/usd_only_routes")
async def data_dummy_everything() -> CvmAllData:    
    return test_generic_linear.create_usd_arbitrage_low_fees_long_path()


@app.get("/data/exchanges/dummy/stables_arbitrage")
async def exchanges_dummy() -> List[AssetPairsXyk[int, int]]:
    """
    example of CVM data registry merged with indexer were we can see stable coin arbitrage
    """
    t1 = new_pair(
        1,
        1,
        2,
        0,
        0,
        1,
        1,
        100,
        20,
        80,
    )
    t2 = new_pair(
        1,
        1,
        2,
        0,
        0,
        1,
        1,
        100,
        30,
        80,
    )
    return [t1, t2]


@app.get("/simulation/ctx")
async def ctx() -> Ctx:
    pass

@app.get("/status")
async def status():
    return {"status": "ok"}


# gets all data from all sources
@app.get("/data/all")
# @cache(expire=3)
async def get_data_all() -> AllData:
    result = get_remote_data()
    return result



@app.post("/simulator/router/data")
def simulator_router_data(data: CvmAllData, input: Input):
    """_summary_
    Given data and input, find and return route.
    """
    ctx = Ctx()
    solution = generic_linear.route(input, data, ctx)
    route = cvxpy_to_data(input, data, ctx, solution)
    from anytree import RenderTree
    result = ""
    for pre, fill, node in RenderTree(route):
        result += format("%s coin=%s/%s" % (pre, node.amount, node.name))
        result += """
        """
    return result 

@app.get("/simulator/router/dummy")
def simulator_dummy():
    return test_generic_linear.test_simulate_all_connected_venues()



@app.get("/data/routable")
# @cache(expire=3)
async def get_data_routable() -> data.AllData:
    result = get_remote_data()

    return result


def get_remote_data() -> AllData:
    cfg = NetworkConfig(
        chain_id="centauri-1",
        url="grpc+" + settings.composable_cosmos_grpc,
        fee_minimum_gas_price=1,
        fee_denomination="ppica",
        staking_denomination="ppica",
    )
    client = LedgerClient(cfg)
    # cvm_contract = LedgerContract(
    #     path=None, client=client, address=settings.cvm_address
    # )

    # cvm_registry_response = cvm_contract.query({"get_config": {}})
    # cvm_registry = GetConfigResponse.parse_obj(cvm_registry_response)
    # skip_api = CosmosChains.parse_raw(
    #     requests.get(settings.skip_money + "v1/info/chains").content
    # )
    osmosis_pools = OsmosisPoolsResponse.parse_raw(
        requests.get(settings.osmosis_pools).content
    )
    # astroport_pools = NeutronPoolsResponse.parse_raw(
    #     requests.get(settings.astroport_pools).content
    # ).result.data
    result = AllData(
        osmosis_pools=osmosis_pools.pools,
        cvm_registry=None,  # cvm_registry,
        astroport_pools=None,  # astroport_pools,
        cosmos_chains=None,  # skip_api,
    )
    return result


def start():
    print(sys.path)
    print(os.environ)
    data.Input(in_token_id=42, out_token_id=42, in_amount=42, out_amount=42, max=True)
    uvicorn.run(
        "main:app",
        host="0.0.0.0",
        port=settings.port,
        reload=True,
        log_level="trace",
        workers=4,
    )


if __name__ == "__main__":
    start()
