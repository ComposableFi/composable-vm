# for alignment on input and output of algorithm

from mantis.simulation.routers.data import (
    Exchange,
    SingleInputAssetCvmRoute,
    Spawn,
)


def test_output_route_centauri_osmosis():
    exchange = Exchange(
        in_asset_amount=100,
        pool_id=1,
        next=[],
        out_asset_amount=42,
        out_asset_id=13,
        in_asset_id=144,
    )

    spawn = Spawn(
        in_asset_amount=100,
        out_asset_id=1,
        in_asset_id=2,
        out_asset_amount=42,
        next=[exchange.model_dump()],
    )
    SingleInputAssetCvmRoute(next=[spawn], input_amount=1000)
