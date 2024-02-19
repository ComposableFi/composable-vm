# given `ExtendedCvmRegistry` and raw `AllData`, and user `Input`, produced oracalized data with assets and venues route level reachable by user

from typing import Union
from blackbox.cvm_runtime.response_to_get_config import GetConfigResponse as CvmRegistry
from blackbox.neutron_pools import Model as NeutronPoolsModel
from blackbox.osmosis_pools import Model as OsmosisPoolsModel
from blackbox.skip_money import Chain
from pydantic import BaseModel
from blackbox.composablefi_networks import Model as NetworksModel




#     def to_cvm():
#         """_summary_
#         prodduce non CVMed XYK
#         prodcue from CVM REG all IN OUT PAIRS of assets in good format
#         filter out only CVMed with known assets
#         replace with ids in XYK
#         generate Transfers for all, find ga for comoss_chain_of each
#         normalize all assets to oracle if ALL assets has USD.

#         Produced CVMed data from raw chain data consumable by simulation
#         """
# ExtendedCvmRegistry
#         pass