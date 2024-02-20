from typing import List

import cachetools
from mantis.blackbox.raw import (
    AllData,
    CosmosChains,
    NeutronPoolsResponse,
    OsmosisPoolsResponse,
)
from cvm_indexer import ExtendedCvmRegistry
from blackbox.settings import settings
from cosmpy.aerial.config import NetworkConfig
from cosmpy.aerial.contract import LedgerClient, LedgerContract
from fastapi import FastAPI
import requests
from blackbox.cvm_runtime.response_to_get_config import GetConfigResponse
from mantis.blackbox import raw
from blackbox.composablefi_networks import Model as NetworksModel
from simulation.routers.angeris_cvxpy import cvxpy_to_data
from simulation.routers import generic_linear
import uvicorn
from simulation.routers import test_generic_linear
from simulation.routers import data
import sys
import os
from simulation.routers.data import (
    AssetPairsXyk,
    Input,
    new_pair,
    read_dummy_data,
    AllData as CvmAllData,
)
from simulation.routers.data import Ctx, read_dummy_data, AllData as CvmAllData
from shelved_cache import PersistentCache
from cachetools import TTLCache

app = FastAPI()

cache = PersistentCache(
    TTLCache, filename="get_remote_data.cache", ttl=120 * 1000, maxsize=2
)


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

    return route


@app.get("/skip_money/chains")
def skip_money_chains():
    raise Exception("For devnet testing to provide hardcoded chains list")


@app.get("/simulator/router/dummy")
def simulator_dummy():
    return test_generic_linear.test_simulate_all_connected_venues()


@app.get("/data/routable/raw")
async def get_data_routable() -> raw.AllData:
    result = get_remote_data()
    return result


@app.get("/data/routable/cvm")
async def get_data_routable() -> ExtendedCvmRegistry:
    raw_data = get_remote_data()
    return ExtendedCvmRegistry(raw_data.cvm_registry, raw_data.networks, raw_data.cosmos_chains, raw_data.osmosis_pools)    

@cachetools.cached(cache, lock=None, info=False)
def get_remote_data() -> AllData:
    cfg = NetworkConfig(
        chain_id=settings.CVM_CHAIN_ID,
        url="grpc+" + settings.CVM_COSMOS_GRPC,
        fee_minimum_gas_price=1,
        fee_denomination=settings.CVM_CHAIN_FEE,
        staking_denomination=settings.CVM_CHAIN_FEE,
    )
    client = LedgerClient(cfg)
    cvm_contract = LedgerContract(
        path=None, client=client, address=settings.cvm_address
    )

    cvm_registry_response = cvm_contract.query({"get_config": {}})
    cvm_registry = GetConfigResponse.parse_obj(cvm_registry_response)
    skip_api = CosmosChains.parse_raw(
        requests.get(settings.skip_money + "v1/info/chains").content
    )
    networks = requests.get(settings.COMPOSABLEFI_NETWORKS).content
    networks = NetworksModel.parse_raw(networks)
    osmosis_pools = OsmosisPoolsResponse.parse_raw(
        requests.get(settings.OSMOSIS_POOLS).content
    )
    astroport_pools = NeutronPoolsResponse.parse_raw(
        requests.get(settings.astroport_pools).content
    ).result.data
    result: AllData = AllData(
        osmosis_pools=osmosis_pools.pools,
        cvm_registry=cvm_registry,
        astroport_pools=astroport_pools,
        cosmos_chains=skip_api,
        networks=networks,
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
