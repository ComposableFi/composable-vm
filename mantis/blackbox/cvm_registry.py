
from typing import List, Union
from blackbox.cvm_runtime.response_to_get_config import GetConfigResponse as CvmRegistry
from blackbox.neutron_pools import Model as NeutronPoolsModel
from blackbox.osmosis_pools import Model as OsmosisPoolsModel
from blackbox.skip_money import Chain
from pydantic import BaseModel
from blackbox.composablefi_networks import Model as NetworksModel, Mainnet


class CvmRegistry:
    """_summary_
    Given on chain and offchain CVM registry data, produce unified view for ease of operations
    """
    cvm_registry: CvmRegistry
    networks : NetworksModel