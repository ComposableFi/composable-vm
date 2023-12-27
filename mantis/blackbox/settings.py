from environs import Env
env = Env()
env.read_env()
from pydantic import BaseModel

class Settings(BaseModel):
    osmosis_pools: str = env("OSMOSIS_POOLS")
    composable_cosmos_grpc : str = env("COMPOSABLE_COSMOS_GRPC")
    cvm_address : str = env("CVM_ADDRESS")
    astroport_pools: str = env("ASTROPORT_POOLS")
    neutron_rpc : str = None
    osmosis_rpc : str = None
    skip_money : str = env("SKIP_MONEY")
    port : int = env("LISTEN_PORT", 8000)

setting = Settings()