from pydantic import Field
from pydantic_settings import BaseSettings

class Settings(BaseSettings):
    osmosis_pools: str | None = Field(alias="OSMOSIS_POOLS", default=None)
    CVM_COSMOS_GRPC: str | None = Field(alias="CVM_COSMOS_GRPC", default="http://grpc.osmosis.zone:9090")
    CVM_CHAIN_ID: str | None = Field(alias="CVM_CHAIN_ID", default="osmosis-1")

    CVM_CHAIN_FEE: str | None = Field(alias="CVM_CHAIN_FEE", default="uosmo")

    cvm_address: str | None = Field(alias="CVM_ADDRESS", default="osmo13guwqtt7xdcuhtewc53tpt9jas5xcnk5tvzdxwhn09774m8jpytqr89pry")
    astroport_pools: str | None = Field(alias="ASTROPORT_POOLS", default=None)
    neutron_rpc: str | None = Field(alias="NEUTRON_RPC", default=None)
    osmosis_rpc: str | None = Field(alias="OSMOSIS_RPC", default=None)
    skip_money: str | None = Field(alias="SKIP_MONEY", default=None)
    port: int = Field(default=8000, alias="LISTEN_PORT")


settings = Settings()
