# given CVM registry and MANTIS offchain registry, and 3rd party indexer/registry data, produce CVM unified view for ease of operations
from typing import List, Optional

from loguru import logger
from pydantic import BaseModel

from blackbox.composablefi_networks import Model as NetworksModel
from blackbox.cvm_runtime.response_to_get_config import (
    AssetId,
    AssetItem,
    AssetReference7,
    ExchangeItem,
    NetworkAssetItem,
    NetworkItem,
    NetworkToNetworkItem,
)
from blackbox.cvm_runtime.response_to_get_config import (
    ExchangeType3 as OsmosisPool,
)
from blackbox.cvm_runtime.response_to_get_config import (
    GetConfigResponse as CvmRegistry,
)
from blackbox.osmosis_pools import Model as OsmosisPoolsModel
from blackbox.skip_money import Chain
from simulation.routers.data import (
    MINIMAL_TRANSACTION_USD_COST_DEFAULT,
    AssetPairsXyk,
    AssetTransfers,
)
from simulation.routers.data import (
    AllData as SimulationData,
)


class ExtendedNetworkItem(NetworkItem):
    gas_price_usd: float
    chain_id: str


class ExtendedExchangeItem(ExchangeItem):
    token_a_amount: int
    token_b_amount: int
    weight_a: float
    weight_b: float
    pool_value_in_usd: float
    fee_per_million: int
    asset_a: AssetItem
    asset_b: AssetItem
    curve: str  # orderbook to be approximated

    @property
    def value_of_a_in_usd(self):
        """
        How much 1 of a costs in USD
        """
        return self.a_usd / self.token_a_amount

    @property
    def value_of_b_in_usd(self):
        """
        How much 1 of a costs in USD
        """
        return self.b_usd / self.token_b_amount

    @property
    def a_usd(self):
        return self.pool_value_in_usd * self.weight_b / (self.weight_a + self.weight_b)

    @property
    def b_usd(self):
        return self.pool_value_in_usd * self.weight_a / (self.weight_a + self.weight_b)

    @property
    def weighted_a(self):
        return self.token_a_amount**self.weight_a

    @property
    def weighted_b(self):
        return self.token_b_amount**self.weight_b

    @property
    def weighted_volume(self):
        return self.weighted_a * self.weighted_b


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

        def find_asset_by_token(token: str) -> Optional[AssetItem]:
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
                indexer = [c for c in indexers_2.root if c.id == str(pool_id)]
                if any(indexer):
                    indexer = indexer[0]
                    token_a = indexer.token0 if indexer.token0 else indexer.pool_assets[0].token.denom
                    token_b = indexer.token1 if indexer.token1 else indexer.pool_assets[1].token.denom
                    asset_a = find_asset_by_token(token_a)
                    asset_b = find_asset_by_token(token_b)

                    if token_a is None or token_b is None:
                        logger.info(
                            "error: mantis::solver::blackbox:: pool has not token denom defined ",
                            pool_id,
                        )
                        continue

                    # raise Exception(indexer)
                    token_a_amount = (
                        int(indexer.token0Amount) if indexer.token0Amount else indexer.pool_assets[0].token.amount
                    )
                    token_b_amount = (
                        int(indexer.token1Amount) if indexer.token1Amount else indexer.pool_assets[1].token.amount
                    )
                    weight_a = 1
                    weight_b = 1
                    fee_per_million = (
                        int(float(indexer.taker_fee) * (1_000_000 / 100))
                        if indexer.taker_fee
                        else int(float(indexer.pool_params.swap_fee) * (1_000_000 / 100))
                    )
                    indexer.scaling_factors
                    x = ExtendedExchangeItem(
                        **onchain.dict(),
                        pool_value_in_usd=indexer.liquidityUsd,
                        token_a_amount=token_a_amount,
                        token_b_amount=token_b_amount,
                        weight_a=weight_a,
                        weight_b=weight_b,
                        fee_per_million=fee_per_million,
                        asset_a=asset_a,
                        asset_b=asset_b,
                        curve="constant_product",
                    )
                    exchanges.append(x)
                else:
                    logger.info(
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


def for_simulation(cvm_registry: ExtendedCvmRegistry, usd_oracles) -> SimulationData:
    """
    Makes data exactly as it handled by simulation
    Optional can use some oracle values from outside
    """
    usd_oracles = {a.asset_id.root: a.usd_per_amount for a in usd_oracles}
    asset_transfers = []
    for transfer in cvm_registry.network_assets:
        asset_transfers.append(
            AssetTransfers(
                in_asset_id=transfer.asset_id.root,
                out_asset_id=transfer.to_asset_id.root,
                in_token_amount=-1,
                out_token_amount=-1,
                venue_fixed_costs_in_usd=0.01,
                fee_per_million=0,
            )
        )
    asset_pairs_xyk = []
    for pair in cvm_registry.exchanges:
        asset_pairs_xyk.append(
            AssetPairsXyk(
                pool_id=int(pair.exchange_id.root),
                in_asset_id=pair.asset_a.asset_id.root,
                out_asset_id=pair.asset_b.asset_id.root,
                fee_of_in_per_million=pair.fee_per_million,
                fee_of_out_per_million=pair.fee_per_million,
                weight_a=pair.weight_a,
                weight_b=pair.weight_b,
                in_token_amount=int(pair.token_a_amount),
                out_token_amount=int(pair.token_b_amount),
                pool_value_in_usd=int(pair.pool_value_in_usd),
                venue_fixed_costs_in_usd=MINIMAL_TRANSACTION_USD_COST_DEFAULT,
                closed=None,
            )
        )

    return SimulationData(
        asset_pairs_xyk=asset_pairs_xyk,
        asset_transfers=asset_transfers,
        usd_oracles=usd_oracles,
    )
