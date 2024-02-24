# oracles which tell price via connections and possible amounts available to go over venues
import copy
from typing import TypeVar, Union

from disjoint_set import DisjointSet

TId = TypeVar("TId", int, str)
TNetworkId = TypeVar("TNetworkId", int, str)
TAmount = TypeVar("TAmount", int, str)


# given set of data about assets, find very approximate ratio of one asset to other
class SetOracle:
    def route(
        partial_oracles: dict[TId, Union[float, None] | None],
        transfers: list[tuple[TId, TId]],
    ) -> dict[TId, Union[float, None] | None]:
        """
        Very fast one and super sloppy on,
        considers if there is connection in general,
        same price and all price everywhere.
        No penalty for high fees/long route and non equilibrium.
        """

        if not partial_oracles:
            return
        print(partial_oracles)
        partial_oracles = copy.deepcopy(partial_oracles)
        ds = DisjointSet()
        for t in transfers:
            ds.union(t[0], t[1])
        for id, value in partial_oracles.items():
            if value is None:
                for other, value in partial_oracles.items():
                    if value and ds.connected(id, other):
                        partial_oracles[id] = value
        return partial_oracles


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

    SetOracle.route(oracles, transfers)
    assert oracles[2] == 2.0
    assert oracles[1] == 1.0
    assert oracles[3] is None
    assert oracles[4] == 2.0
