import os
import sys
from typing import List

import cachetools
import requests
import uvicorn
from cachetools import TTLCache
from cosmpy.aerial.config import NetworkConfig
from cosmpy.aerial.contract import LedgerClient, LedgerContract
from cvm_indexer import ExtendedCvmRegistry, for_simulation
from fastapi import Depends, FastAPI
from loguru import logger
from shelved_cache import PersistentCache

from blackbox import raw
from blackbox.composablefi_networks import Model as NetworksModel
from blackbox.cvm_runtime.response_to_get_config import GetConfigResponse
from blackbox.raw import (
    AllData,
    CosmosChains,
    NeutronPoolsResponse,
    OsmosisPoolsResponse,
)
from blackbox.settings import settings
from simulation.routers import data, generic_linear, test_generic_linear
from simulation.routers.angeris_cvxpy import cvxpy_to_data
from simulation.routers.data import (
    AllData as SimulationData,
)
from simulation.routers.data import (
    AssetPairsXyk,
    Ctx,
    Input,
    SingleInputAssetCvmRoute,
    new_pair,
    read_dummy_data,
)
from simulation.routers.oracles import bforacle
from simulation.routers.scaler import scale_in

""""
logger = logging.getLogger(__name__)

def create_app() -> FastAPI:
    app = FastAPI(title="CustomLogger", debug=False)
    logger = CustomizeLogger.make_logger()
    app.logger = logger

    return app


app = create_app()
"""
app = FastAPI()

cache = PersistentCache(TTLCache, filename="get_remote_data.cache", ttl=12 * 1000, maxsize=2)


# 1. return csv data + data schema in 127.0.0.1:8000/docs
# https://github.com/ComposableFi/composable/issues/4434
@app.get("/data/dummy/everything")
async def data_dummy_everything() -> SimulationData:
    data = read_dummy_data("./simulation/routers/data/")
    return data


@app.get("/data/dummy/usd_only_routes")
async def data_dummy_usd_only_routes() -> SimulationData:
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


@app.post("/simulator/router/dummy")
def simulator_router_dummy(data: SimulationData, input: Input):
    """_summary_
    Given data and input, find and return route.
    """
    ctx = Ctx()
    solution = generic_linear.route(input, data, ctx)
    route = cvxpy_to_data(input, data, ctx, solution)

    return route


@app.get("/simulator/router")
def simulator_router(input: Input = Depends()) -> list[SingleInputAssetCvmRoute]:
    """_summary_
    Given input, find and return route.
    """
    raw_data = get_remote_data()
    cvm_data = ExtendedCvmRegistry.from_raw(
        raw_data.cvm_registry,
        raw_data.networks,
        raw_data.cosmos_chains.chains,
        raw_data.osmosis_pools,
    )

    route = solve(input, cvm_data)

    return route


def solve(original_input: Input, cvm_data: ExtendedCvmRegistry) -> list[SingleInputAssetCvmRoute]:
    ctx = Ctx()
    original_data = for_simulation(cvm_data, {})

    original_input.in_amount = int(original_input.in_amount)
    original_input.out_amount = int(original_input.out_amount)

    if original_input.in_amount >= ctx.max_trade * original_data.maximal_reserves_of(original_input.in_token_id):
        raise Exception(
            f"you are trading on market limit with {original_input.in_amount} for {original_data.maximal_reserves_of(original_input.in_token_id)}"
        )

    result = bforacle.route(original_input, original_data, ctx, max_depth=6, splits=1, revision=True)
    raise Exception(result)

    scaled_data, scaled_input, scale = scale_in(original_data, original_input, ctx)
    solutions = generic_linear.route(scaled_input, scaled_data, ctx)
    routes = cvxpy_to_data(original_input, original_data, ctx, solutions, scale)
    routes = [route.lower() for route in routes]
    return routes


@app.get("/skip_money/chains")
def skip_money_chains():
    raise Exception("For devnet testing to provide hardcoded chains list")


@app.get("/simulator/router/random")
def simulator_dummy():
    return test_generic_linear.test_simulate_all_connected_venues()


@app.get("/data/routable/raw")
async def get_data_routable() -> raw.AllData:
    result = get_remote_data()
    return result


@app.get("/data/routable/cvm")
async def get_data_routable_cvm() -> ExtendedCvmRegistry:
    raw_data = get_remote_data()
    return ExtendedCvmRegistry.from_raw(
        raw_data.cvm_registry,
        raw_data.networks,
        raw_data.cosmos_chains.chains,
        raw_data.osmosis_pools,
    )


@app.get("/data/routable/oracalized_cvm")
async def get_data_routable_oracalized() -> SimulationData:
    raw_data = get_remote_data()
    cvm_data = ExtendedCvmRegistry.from_raw(
        raw_data.cvm_registry,
        raw_data.networks,
        raw_data.cosmos_chains.chains,
        raw_data.osmosis_pools,
    )
    return for_simulation(cvm_data, {})


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
    cvm_contract = LedgerContract(path=None, client=client, address=settings.cvm_address)

    cvm_registry_response = cvm_contract.query({"get_config": {}})
    cvm_registry = GetConfigResponse.parse_obj(cvm_registry_response)
    skip_api = CosmosChains.parse_raw(requests.get(settings.skip_money + "v1/info/chains").content)
    networks = requests.get(settings.COMPOSABLEFI_NETWORKS).content
    networks = NetworksModel.parse_raw(networks)
    osmosis_pools = OsmosisPoolsResponse.parse_raw(requests.get(settings.OSMOSIS_POOLS).content)
    astroport_pools = NeutronPoolsResponse.parse_raw(requests.get(settings.astroport_pools).content).result.data
    result: AllData = AllData(
        osmosis_pools=osmosis_pools.pools,
        cvm_registry=cvm_registry,
        astroport_pools=astroport_pools,
        cosmos_chains=skip_api,
        networks=networks,
    )

    return result


def start():
    logger.info(sys.path)
    logger.info(os.environ)
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
