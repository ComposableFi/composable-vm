# given CVM registry and MANTIS offchain registry, and 3rd party indexer/registry data, produce CVM unified view for ease of operations

from typing import List, Optional

from blackbox.cvm_runtime.response_to_get_config import (
    AssetItem,
    AssetReference7,
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


class ExtendedExchangeItem(ExchangeItem):
    token_a_amount: int
    token_b_amount: int
    weight_a: float
    weight_b: float
    fee_per_million: int
    asset_a : AssetItem
    asset_b : AssetItem
class ExtendedCvmRegistry(BaseModel):
    """_summary_
    Given on chain and offchain CVM registry data, produce unified view for ease of operations
    """

    assets: List[AssetItem]
    exchanges: List[ExtendedExchangeItem]
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
            if any(static):  # not removed from static routing
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
                    chain_id=static.CHAIN_ID,
                    gas_price_usd=gas_price_usd,
                    **onchain.dict(),
                    )
                networks.append(x)

        assets = onchains.assets
        
        def find_asset_by_token(token: str) -> Optional[AssetItem] :
            for asset in assets:
                local: AssetReference7 = asset.local.root
                if local.native.denom == token:
                    return asset
            return None
        
        exchanges = []
        for onchain in onchains.exchanges:
            if isinstance(onchain.exchange.root, OsmosisPool):
                subonchain: OsmosisPool = onchain.exchange.root
                pool_id = subonchain.osmosis_pool_manager_module_v1_beta1.pool_id
                print("pool_id", pool_id)
                indexer = [c for c in indexers_2.root if c.id == str(pool_id)]
                if any(indexer):
                    indexer = indexer[0]

                    print(indexer)
                    token_a = indexer.token0 if indexer.token0 else indexer.pool_assets[0].token.denom
                    token_b = indexer.token1 if indexer.token1 else indexer.pool_assets[1].token.denom
                    asset_a = find_asset_by_token(token_a)
                    asset_b = find_asset_by_token(token_b)
                    
                    if token_a is None or token_b is None:
                        print(
                            "error: mantis::solver::blackbox:: pool has not token denom defined ",
                            pool_id,
                        )
                        continue
                    
                    # raise Exception(indexer)
                    token_a_amount = (
                        int(indexer.token0Amount)
                        if indexer.token0Amount
                        else indexer.pool_assets[0].token.amount
                    )
                    token_b_amount = (
                        int(indexer.token1Amount)
                        if indexer.token1Amount
                        else indexer.pool_assets[1].token.amount
                    )
                    weight_a = 1
                    weight_b = 1
                    fee_per_million = int(
                        float(indexer.taker_fee) * (1_000_000 / 100) 
                    ) if indexer.taker_fee else int(
                        float(indexer.pool_params.swap_fee) * (1_000_000 / 100)
                    ) 
                    indexer.scaling_factors
                    x = ExtendedExchangeItem(
                        **onchain.dict(),                        
                        liquidity_usd=indexer.liquidityUsd,
                        token_a_amount=token_a_amount,
                        token_b_amount=token_b_amount,
                        weight_a=weight_a,
                        weight_b=weight_b,
                        fee_per_million=fee_per_million,
                        asset_a = asset_a,
                        asset_b = asset_b
                    )
                    exchanges.append(x)
                else:
                    print(
                        "error: mantis::solver::blackbox:: no pool indexer info found for ",
                        pool_id,
                    )

        network_assets = onchains.network_assets
        network_to_networks = onchains.network_to_networks
        return cls(
            assets=assets,
            exchanges=exchanges,
            network_assets=network_assets,
            network_to_networks=network_to_networks,
            networks=networks,
        )


class Oracle(BaseModel):
    # given `ExtendedCvmRegistry` and raw `AllData`, and user `Input`, produced oracalized data with assets and venues route level reachable by user
    def from_usd(cvm: ExtendedCvmRegistry):
        """_summary_
            Builds USD oracle from data.
        """
        all_assets = [a.asset_id.root for a in cvm.assets]
        oracle = [1] * len(all_assets)
        for asset in all_assets:
            for exchange in cvm.exchanges:
                if isinstance(exchange.exchange, OsmosisPool):
                    pool : OsmosisPool = exchange.exchange          
        pass
    
    def for_simulation():
        """
        Makes data exactly as it handled by simulation
        """
        pass