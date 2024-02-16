from typing import Union
from blackbox.cvm_runtime.response_to_get_config import GetConfigResponse as CvmRegistry
from blackbox.neutron_pools import Model as NeutronPoolsModel
from blackbox.osmosis_pools import Model as OsmosisPoolsModel
from blackbox.skip_money import Chain
from pydantic import BaseModel
from blackbox.composablefi_networks import Model as NetworksModel

class OsmosisPoolsResponse(BaseModel):
    pools: OsmosisPoolsModel


class CosmosChains(BaseModel):
    chains: list[Chain]


class NeutronPoolsResponseData(BaseModel):
    data: NeutronPoolsModel


class NeutronPoolsResponse(BaseModel):
    result: NeutronPoolsResponseData


class AllData(BaseModel):
    astroport_pools: Union[NeutronPoolsModel, None]
    osmosis_pools: Union[OsmosisPoolsModel, None]
    cvm_registry: CvmRegistry
    cosmos_chains: CosmosChains
    networks : NetworksModel
