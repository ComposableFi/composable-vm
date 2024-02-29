# oracles which tell price via connections and possible amounts available to go over venues
import copy
from typing import TypeVar, Union

from disjoint_set import DisjointSet

TId = TypeVar("TId", int, str)
TNetworkId = TypeVar("TNetworkId", int, str)
TAmount = TypeVar("TAmount", int, str)


def merge_by_connection_from_existing(
    oracles: dict[TId, Union[float, None] | None],
    transfers: list[tuple[TId, TId]],
) -> dict[TId, Union[float, None] | None]:
    """
    given set of data about assets, find very approximate ratio of one asset to other
    Very fast one and super sloppy on,
    considers if there is connection in general,
    same price and all price everywhere.
    No penalty for high fees/long route and non equilibrium.
    """

    if not oracles or len(oracles) == 0:
        oracles = {}
    oracles = copy.deepcopy(oracles)
    ds = DisjointSet()
    all_asset_ids = set()
    for t in transfers:
        ds.union(t[0], t[1])
        all_asset_ids.add(t[0])
        all_asset_ids.add(t[1])
    for asset_id in oracles.keys():
        all_asset_ids.add(asset_id)
    for asset_id in all_asset_ids:
        if asset_id not in oracles or oracles[asset_id] is None or oracles[asset_id] == 0:
            for other_id, value in oracles.items():
                if value and ds.connected(asset_id, other_id) and value > 0:
                    oracles[asset_id] = value
                    break
    return oracles


def test():
    oracles = {
        1: 1.0,
        2: None,
        3: None,
        4: 2.0,
    }
    transfers = [
        (2, 4),
        (1, 2),
    ]

    merge_by_connection_from_existing(oracles, transfers)
    assert oracles[2] == 2.0
    assert oracles[1] == 1.0
    assert oracles[3] is None
    assert oracles[4] == 2.0
