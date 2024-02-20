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
    gas_price: int
    chain_id: str
    pass


class ExtendedExchageItem(ExchangeItem):
    token_a_amount: int
    token_b_amount: int
    weight_a: float
    weight_b: float
    fee_per_million: int
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

    def __init__(
        self,
        onchains: CvmRegistry,
        statics: NetworksModel,
        indexers_1: list[Chain],
        indexers_2: OsmosisPoolsModel,
    ):
        super().__init__()
        statics = [statics.pica.mainnet, statics.osmosis.mainnet]
        self.networks = []
        for onchain in onchains.networks:
            static = [x for x in statics if x.NETWORK_ID == onchain.network_id][0]
            indexer = [c for c in indexers_1 if c.chain_id == static.CHAIN_ID][0]
            gas_price = int(indexer.fee_assets[0].gas_price_info.high)
            x = ExtendedNetworkItem(
                **onchain, chain_id=static.CHAIN_ID, gas_price=gas_price
            )
            self.networks.append(x)

        self.exchanges = []
        for onchain in onchains.exchanges:
            if isinstance(onchain.exchange.exchange_type.root, OsmosisPool):
                subonchain: OsmosisPool = onchain.exchange.exchange_type.root
                pool_id = subonchain.osmosis_pool_manager_module_v1_beta1.pool_id
                indexer = [c for c in indexers_2.root if c.id == pool_id][0]
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
                self.exchanges.append(x)

        self.assets = onchains.assets
        self.network_assets = onchains.network_assets
        self.network_to_networks = onchains.network_to_networks
