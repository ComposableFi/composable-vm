# for alignment on input and output of algorithm
from pathlib import Path

from mantis.simulation.routers.data import (
    Exchange,
    SingleInputAssetCvmRoute,
    Spawn,
    new_data,
    new_pair,
    read_dummy_data,
)

TEST_DATA_DIR = Path(__file__).resolve().parent.as_posix() + "/data/"


def test_all_data_from_csv():
    assert read_dummy_data(TEST_DATA_DIR)


def test_usd_price():
    pica_usd = new_pair(
        1,
        1,
        2,
        0,
        0,
        1,
        1,
        100,
        20,
        80,
    )
    data = new_data([pica_usd], [])
    price = data.token_price_in_usd(2)
    assert price == 0.625
    assert data.all_tokens == [1, 2]
    assert data.venues_count == 1
    assert data.index_of_token(1) == 0
    assert data.index_of_token(2) == 1


def test_output_route_centauri_osmosis():
    exchange = Exchange(in_asset_amount=100, pool_id=1, next=[])

    spawn = Spawn(
        in_asset_amount=100,
        out_asset_id=1,
        next=[exchange.model_dump()],
    )
    SingleInputAssetCvmRoute(start=[spawn])
