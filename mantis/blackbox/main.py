from fastapi import FastAPI
from cosmpy.aerial.contract import LedgerClient
from cosmpy.aerial.config import NetworkConfig
from cosmpy.cosmwasm.rest_client import CosmWasmRestClient
import requests

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

import blackbox.cvm_runtime.query as cvm

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
    url="grpc+http://composable-grpc.polkachu.com:22290",
    fee_minimum_gas_price=1,
    fee_denomination="ppica",
    staking_denomination="ppica",
    )
    
    client = LedgerClient(cfg)
    wasm : CosmWasmRestClient = client.wasm
    response: QueryAllContractStateResponse = wasm.AllContractState(QueryAllContractStateRequest(address="centauri1lkh7p89tdhkc52vkza5jus5xmgjqjut6ngucsn88mhmzaqc02h5qu89k2u"))
    print(response)
    
    result = {}
    result["cvm"] = response.models[1].value
    result["pools"] = requests.get("https://app.osmosis.zone/api/pools?page=1&limit=1000&min_liquidity=500000").text
    return result