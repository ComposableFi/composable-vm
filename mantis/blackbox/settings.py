from pydantic import BaseModel, Field
from pydantic_settings import BaseSettings, SettingsConfigDict

class Settings(BaseSettings):
    osmosis_pools: str = Field(alias="OSMOSIS_POOLS")
    composable_cosmos_grpc : str = Field(alias="COMPOSABLE_COSMOS_GRPC")
    cvm_address : str = Field(alias="CVM_ADDRESS")
    astroport_pools: str = Field(alias="ASTROPORT_POOLS")
    neutron_rpc : str = None
    osmosis_rpc : str = None
    skip_money : str = Field(alias="SKIP_MONEY")
    port : int = Field(default= 8000, alias="LISTEN_PORT")

setting = Settings()