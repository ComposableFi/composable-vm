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
import uvicorn
from simulation.routers import test_generic_linear
from simulation.routers import data
import sys
import os
from pydantic import BaseModel
from simulation.routers.data import (
    AssetPairsXyk,
    new_pair,
    read_dummy_data,
    AllData as CvmAllData,
)
from simulation.routers.data import Ctx, read_dummy_data, AllData as CvmAllData

app = FastAPI()


# 1. return csv data + data schema in 127.0.0.1:8000/docs
# https://github.com/ComposableFi/composable/issues/4434
@app.get("/xyk_pairs")
async def xyk_pairs() -> CvmAllData[int, int]:
    data = read_dummy_data("./simulation/routers/data/")
    return data


@app.get("/exchanges/dummy/stables_arbitrage")
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


class White_csv_Data(BaseModel):
    pool_id: int
    in_asset_id: int
    out_asset_id: int
    fee_of_in_per_million: int
    fee_of_out_per_million: int
    weight_of_a: int
    weigth_of_b: int
    pool_value_in_usd: int
    in_token_amount: int
    out_token_amount: int


@app.get("/status")
async def status():
    return {"status": "ok"}


# gets all data from all sources
@app.get("/data/all")
# @cache(expire=3)
async def get_data_all() -> AllData:
    result = get_data()
    return result


@app.get("/data/routable")
# @cache(expire=3)
async def get_data_routable() -> data.AllData:
    result = get_data()

    return result


def get_data() -> AllData:
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


@app.get("/simulator/dummy")
def simulator_dummy():
    return test_generic_linear.test_simulate_all_connected_venues()


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
