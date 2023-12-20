from fastapi import FastAPI
from cosmpy.aerial.contract import LedgerClient
from cosmpy.aerial.config import NetworkConfig
from cosmpy.cosmwasm.rest_client import CosmWasmRestClient
import requests
import json
from blackbox.osmosis_pools import Model as OsmosisPoolsModel

from cosmpy.protos.cosmwasm.wasm.v1.query_pb2 import (
    QueryAllContractStateRequest,
    QueryAllContractStateResponse,
    QueryCodeRequest,
    QueryCodeResponse,
    QueryCodesRequest,
    QueryCodesResponse,
    QueryContractHistoryRequest,
    QueryContractHistoryResponse,
    QueryContractInfoRequest,
    QueryContractInfoResponse,
    QueryContractsByCodeRequest,
    QueryContractsByCodeResponse,
    QueryRawContractStateRequest,
    QueryRawContractStateResponse,
    QuerySmartContractStateRequest,
    QuerySmartContractStateResponse,
)

from blackbox.settings import setting
import blackbox.cvm_runtime.query as cvm
from blackbox.models import AllData, OsmosisPoolsResponse

app = FastAPI()

@app.get("/status")
async def status():
    return {"status": "ok"}

@app.get("/assets/{id}")
async def id(id: str):
    return {"asset_id": cvm.AssetId(__root__= id)}


# gets all data from all sources
@app.get("/data/all")
async def get_data_all():
    
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
    wasm : CosmWasmRestClient = client.wasm
    response: QueryAllContractStateResponse = wasm.SmartContractState(QuerySmartContractStateRequest(address="centauri1lkh7p89tdhkc52vkza5jus5xmgjqjut6ngucsn88mhmzaqc02h5qu89k2u"), )
    print(response)
    
    # result = {}
    # result["cvm"] = response.models[1].value
    
    osmosis_pools = OsmosisPoolsResponse.parse_raw(requests.get(setting.osmosis_pools).content)
    result = AllData(osmosis_pools = osmosis_pools.pools)
    return result