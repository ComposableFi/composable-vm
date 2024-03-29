from typing import Union

from pydantic import BaseModel

from blackbox.composablefi_networks import Model as NetworksModel
from blackbox.cvm_runtime.response_to_get_config import GetConfigResponse as CvmRegistry
from blackbox.neutron_pools import Model as NeutronPoolsModel
from blackbox.osmosis_pools import Model as OsmosisPoolsModel
from blackbox.skip_money import Chain


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
    cosmos_chains: CosmosChains
    cvm_registry: CvmRegistry
    networks: NetworksModel
