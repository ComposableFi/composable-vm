from fastapi import FastAPI

from cosmpy import cosm

import blackbox.cvm_runtime.query as cvm

app = FastAPI()

@app.get("/status")
async def status():
    return {"status": "ok"}

@app.get("/assets/{id}")
async def id(id: str):
    return {"asset_id": cvm.AssetId(__root__= id)}
