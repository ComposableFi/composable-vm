# given CVM registry and MANTIS offchain registry, and 3rd party indexer/registry data, produce CVM unified view for ease of operations

from typing import List
from blackbox.cvm_runtime.response_to_get_config import (
    AssetItem,
    ExchangeItem,
    GetConfigResponse as CvmRegistry,
    NetworkAssetItem,
    NetworkItem,
    NetworkToNetworkItem,
    ExchangeType3 as OsmosisPool,
)
from blackbox.osmosis_pools import Model as OsmosisPoolsModel
from blackbox.skip_money import Chain
from pydantic import BaseModel
from blackbox.composablefi_networks import Model as NetworksModel


class ExtendedNetworkItem(NetworkItem):
    gas_price_usd: float
    chain_id: str


class ExtendedExchageItem(ExchangeItem):
    token_a_amount: int
    token_b_amount: int
    weight_a: float
    weight_b: float
    fee_per_million: int


class ExtendedCvmRegistry(BaseModel):
    """_summary_
    Given on chain and offchain CVM registry data, produce unified view for ease of operations
    """

    assets: List[AssetItem]
    exchanges: List[ExchangeItem]
    network_assets: List[NetworkAssetItem]
    network_to_networks: List[NetworkToNetworkItem]
    networks: List[ExtendedNetworkItem]

    @classmethod  
    def from_raw(
        cls,
        onchains: CvmRegistry,
        statics: NetworksModel,
        indexers_1: list[Chain],
        indexers_2: OsmosisPoolsModel,
    ):        
        statics = [statics.pica.mainnet, statics.osmosis.mainnet]
        networks = []
        for onchain in onchains.networks:
            print("================================")
            print(onchain)
            print(statics)
            static = [x for x in statics if x.NETWORK_ID == onchain.network_id.root]
            if any(static): # not removed from static routing
                static = static[0]
                indexer = [c for c in indexers_1 if c.chain_id == static.CHAIN_ID][0]
                def indexer_1_gas_to_cvm(indexer):
                    if any(indexer.fee_assets):
                        gas = indexer.fee_assets[0].gas_price_info
                        if gas:
                            if gas.high:
                                return float(gas.high)                    
                            if gas.average:
                                return float(gas.average)
                            if gas.low:
                                return float(gas.low)
                    return 0.0
                gas_price_usd = indexer_1_gas_to_cvm(indexer)
                x = ExtendedNetworkItem(
                    # onchain
                    chain_id=static.CHAIN_ID, 
                    gas_price_usd=gas_price_usd,
                    accounts = onchain.accounts,
                    network_id = onchain.network_id,
                    ibc = onchain.ibc,
                    outpost = onchain.outpost,
                )                
                networks.append(x)
        
        exchanges = []
        for onchain in onchains.exchanges:
            if isinstance(onchain.exchange.root, OsmosisPool):
                subonchain: OsmosisPool = onchain.exchange.root
                pool_id = subonchain.osmosis_pool_manager_module_v1_beta1.pool_id
                print("pool_id", pool_id)                
                indexer = [c for c in indexers_2.root if c.id == str(pool_id)][0]
                token_a_amount = indexer.token0Amount
                token_b_amount = indexer.token1Amount
                weight_a = 1
                weight_b = 1
                fee_per_million = indexer.pool_params.swap_fee * (1_000_000 / 100)
                indexer.scaling_factors
                x = ExtendedExchageItem(
                    **onchain,
                    liquidity_usd=indexer.liquidityUsd,
                    token_a_amount=token_a_amount,
                    token_b_amount=token_b_amount,
                    weight_a=weight_a,
                    weight_b=weight_b,
                    fee_per_million=fee_per_million,
                )
                exchanges.append(x)

        assets = onchains.assets
        network_assets = onchains.network_assets
        network_to_networks = onchains.network_to_networks
        return cls(
            assets=assets,
            exchanges=exchanges,
            network_assets=network_assets,
            network_to_networks=network_to_networks,
            networks=networks,
        )
