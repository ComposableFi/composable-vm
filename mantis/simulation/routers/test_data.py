# for alignment on input and output of algorithm
from pathlib import Path

from simulation.routers.data import (
    new_data,
    new_pair,
    new_transfer,
    read_dummy_data,
)

TEST_DATA_DIR = Path(__file__).resolve().parent.as_posix() + "/data/"


def _test_all_data_from_csv():
    assert read_dummy_data(TEST_DATA_DIR)


def _test_token_price_in_usd():
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


def test_token_price_in_usd_via_oracle():
    pica_usd = new_pair(
        1,
        1,
        2,
        0,
        0,
        1,
        1,
        100,
        10,
        90,
    )
    data = new_data([pica_usd], [], {1: None, 2: None})
    if data.asset_pairs_xyk is None:
        raise ValueError("asset_pairs_xyk can't be None")
    assert data.asset_pairs_xyk[0].a_usd == 50
    assert data.asset_pairs_xyk[0].b_usd == 50
    assert data.asset_pairs_xyk[0].value_of_a_in_usd == data.token_price_in_usd(1)
    assert data.token_price_in_usd(1) == 5
    assert data.token_price_in_usd(2) == 0.5555555555555556
    assert data.all_tokens == [1, 2]
    assert data.venues_count == 1
    assert data.index_of_token(1) == 0
    assert data.index_of_token(2) == 1


def test_transfer_to_exchange():
    connection12 = new_transfer(1, 2, 100, 1, 2, 100)
    connection23 = new_transfer(2, 3, 100, 1, 2, 100)
    connection45 = new_transfer(4, 5, 100, 1, 2, 100)
    pair34 = new_pair(1, 3, 2, 0, 0, 1, 1, 100, 20, 80)
    pair12 = new_pair(2, 1, 2, 0, 0, 1, 1, 100, 500, 80)
    pair43 = new_pair(3, 4, 3, 0, 0, 1, 1, 100, 500, 80)
    data = new_data([pair12, pair34, pair43], [connection12, connection23, connection45])

    route1 = data.transfer_to_exchange(1)
    route2 = data.transfer_to_exchange(2)
    route3 = data.transfer_to_exchange(3)
    route5 = data.transfer_to_exchange(5)

    assert len(route1) == 3
    assert len(route2) == 3
    assert len(route3) == 3
    assert len(route5) == 1
