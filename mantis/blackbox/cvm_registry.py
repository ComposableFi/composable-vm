
from typing import List, Union
from blackbox.cvm_runtime.response_to_get_config import AssetItem, ExchangeItem, GetConfigResponse as CvmRegistry, NetworkAssetItem, NetworkItem, NetworkToNetworkItem
from blackbox.neutron_pools import Model as NeutronPoolsModel
from blackbox.osmosis_pools import Model as OsmosisPoolsModel
from blackbox.skip_money import Chain
from mantis.simulation.routers.data import AssetTransfers
from pydantic import BaseModel
from blackbox.composablefi_networks import Model as NetworksModel, Mainnet


class ExtendedNetworkItem(NetworkItem):
    gas_price : int
    chain_id : str
    pass

class ExtendedExchageItem(ExchangeItem):
    # fee, weight, etc
    pass



class ExtendedCvmRegistry(BaseModel):
    """_summary_
    Given on chain and offchain CVM registry data, produce unified view for ease of operations
    """
    assets: List[AssetItem]
    exchanges: List[ExchangeItem]
    network_assets: List[NetworkAssetItem]
    network_to_networks: List[NetworkToNetworkItem]
    networks: List[ExtendedNetworkItem]
    
    def __init__(self, onchains: CvmRegistry, statics: NetworksModel, indexers_1: list[Chain]):
        super().__init__()
        statics = [statics.pica.mainnet, statics.osmosis.mainnet]
        self.networks = []
        for onchain in onchains.networks:
            static = [x for x in statics if x.NETWORK_ID == onchain.network_id][0]
            indexer = [c for c in indexers_1 if c.chain_id == static.CHAIN_ID][0]
            gas_price = int(indexer.fee_assets[0].gas_price_info.high)
            x = ExtendedNetworkItem(**onchain, chain_id = static.CHAIN_ID, gas_price = gas_price)
            self.networks.append(x)

        self.exchanges = onchains.exchanges # merge with Osmossi pool data
        
        self.assets = onchains.assets
        self.network_assets = onchains.network_assets
        self.network_to_networks = onchains.network_to_networks
        