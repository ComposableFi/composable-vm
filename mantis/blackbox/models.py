from typing import List, Union
from blackbox.cvm_runtime.response_to_get_config import GetConfigResponse as CvmRegistry
from blackbox.neutron_pools import Model as NeutronPoolsModel
from blackbox.osmosis_pools import Model as OsmosisPoolsModel
from blackbox.skip_money import Chain
from pydantic import BaseModel
from blackbox.composablefi_networks import Model as NetworksModel, Mainnet

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
    



    
    def to_cvm(self):
        """_summary_
        prodduce non CVMed XYK
        prodcue from CVM REG all IN OUT PAIRS of assets in good format
        filter out only CVMed with known assets
        replace with ids in XYK
        generate Transfers for all, find ga for comoss_chain_of each
        normalize all assets to oracle if ALL assets has USD.
        
        Produced CVMed data from raw chain data consumable by simulation
        """
        
        # for each chain find all transfers
        for net in self.nets:
            
        
        pass 