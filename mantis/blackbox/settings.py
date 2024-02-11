from pydantic import Field
from pydantic_settings import BaseSettings


class Settings(BaseSettings):
    osmosis_pools: str | None = Field(alias="OSMOSIS_POOLS", default=None)
    composable_cosmos_grpc: str | None = Field(
        alias="COMPOSABLE_COSMOS_GRPC", default=None
    )
    cvm_address: str | None = Field(alias="CVM_ADDRESS", default=None)
    astroport_pools: str | None = Field(alias="ASTROPORT_POOLS", default=None)
    neutron_rpc: str | None = Field(alias="NEUTRON_RPC", default=None)
    osmosis_rpc: str | None = Field(alias="OSMOSIS_RPC", default=None)
    skip_money: str | None = Field(alias="SKIP_MONEY", default=None)
    port: int = Field(default=8000, alias="LISTEN_PORT")


settings = Settings()
