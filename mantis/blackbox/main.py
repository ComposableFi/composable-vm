from fastapi import FastAPI

import blackbox.cvm_route as cvm

app = FastAPI()

@app.get("/status")
async def status():
    return {"status": "ok"}

@app.get("/assets/{id}")
async def id(id: str):
    return {"asset_id": cvm.AssetId(__root__= id)}