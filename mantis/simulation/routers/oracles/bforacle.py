import copy
import decimal
import math
from dataclasses import dataclass

from loguru import logger

from simulation.routers.data import (
    AllData,
    AssetPairsXyk,
    AssetTransfers,
    Ctx,
    Exchange,
    Input,
    SingleInputAssetCvmRoute,
    Spawn,
    TAmount,
    TId,
)
from simulation.routers.errors import Infeasible


class Edge:
    # A class that represent an edge in an useful way
    # In the future it could be used in hypergraph algorithms with a few changes

    venue: AssetTransfers | AssetPairsXyk

    nodes: list[int]
    """
    nodes of the edge
    """

    balances: list[TAmount]
    """
    amount of each token in the edge
    """

    weights: list[int]
    """
    weight of each token in the edge
    """
    fees: list[float]
    """
    fee of each token in the edge
    """
    constant_fees: list[float]
    """
    constant fee of each token in the edge
    """

    def toFloatOrZero(self, x):  # Cast to float using 0.0 when it fails
        return float(x) if x else 0.0

    def __init__(
        self,
        e: list[AssetTransfers | AssetPairsXyk],
        tokensIds: dict[TId, int],
        usd_oracles: dict[TId, int],
    ):
        # Creates an edge in base to a pool
        if isinstance(e, AssetTransfers):
            self.__initFromTransfers(e, tokensIds, usd_oracles)
        else:
            self.__initFromPairsXyk(e, tokensIds, usd_oracles)

    def __initFromTransfers(self, venue: AssetTransfers, tokensIds: dict[TId, int], usd_oracles: dict[TId, int]):
        self.nodes = [tokensIds[venue.in_asset_id], tokensIds[venue.out_asset_id]]
        # raise Exception("transfer amounts are not subject to same logic as exchanges")
        self.balances = [10**22, 10**22]  # [e.in_token_amount, e.out_token_amount]
        self.weights = [1, 1]
        self.venue = copy.deepcopy(venue)
        self.fees = [
            float(venue.fee_per_million) / 1_000_000.0,
            float(venue.fee_per_million) / 1_000_000.0,
        ]
        self.constant_fees = [0, 0]

    def __initFromPairsXyk(self, venue, tokensIds: dict[TId, int], usd_oracles: dict[TId, int]):
        self.nodes = [tokensIds[venue.in_asset_id], tokensIds[venue.out_asset_id]]
        self.balances = [venue.in_asset_amount, venue.out_asset_amount]
        self.venue = copy.deepcopy(venue)
        self.weights = [venue.weight_a, venue.weight_b]
        self.fees = [self.toFloatOrZero(venue.fee_in), self.toFloatOrZero(venue.fee_out)]
        self.constant_fees = [0, 0]

    def trade(self, asset_index, in_asset_amount):
        decimal.getcontext().prec = 28
        # Actually do the change of the amount of the tokens
        i, o = 0, 1
        if asset_index == self.nodes[1]:
            i, o = 1, 0
        in_asset_amount = (in_asset_amount - self.constant_fees[i]) * (1 - self.fees[i])
        result = decimal.Decimal(self.balances[o]) * (
            1
            - (
                decimal.Decimal(self.balances[i])
                / (decimal.Decimal(self.balances[i]) + decimal.Decimal(in_asset_amount))
            )
            ** decimal.Decimal(self.weights[i] / self.weights[o])
        )
        result = float(result)

        self.balances[i] += in_asset_amount
        self.balances[o] -= result
        return result

    def other(self, Ti):  # Assumes only 2 nodes in the edge
        if Ti == self.nodes[0]:
            return self.nodes[1]
        return self.nodes[0]

    def __repr__(self):
        return f"Edge(a={self.nodes}, b={self.balances}, w={self.weights}, f={self.fees}, cf={self.constant_fees}, {self.venue})"


@dataclass
class Previous:
    used_venue_index: int | None
    """
    Used venue index to arrive to `amount`
    """
    amount: int

    def default(self) -> "Previous":
        return Previous(None, 0)

    def start(self, tendered_asset) -> "Previous":
        return Previous(None, tendered_asset)


class State:
    # A class that represent the state of the algorithm
    # It's used to pass the state to the threads if executed in parallel
    max_depth: int
    depth: int
    distances: list[list[Previous]]
    received_asset_index: int
    edges: list[Edge]
    j: int
    n: int

    def reset_distances(self, max_depth):
        self.distances = [[Previous.default()] * (max(max_depth) + 1) for _ in range(self.n)]

    def __init__(self):
        self.distances = None
        self.max_depth = None
        self.depth = None
        self.received_asset_index = None
        self.edges = None
        self.j = None
        self.n = None


def data2bf(
    all_data: AllData,
):
    edges: list[Edge] = []
    all_tokens = all_data.all_tokens
    asset_id_to_index = {x: i for i, x in enumerate(all_tokens)}
    index_to_asset_id = {i: x for i, x in enumerate(all_tokens)}
    for x in all_data.asset_transfers:
        edges.append(Edge(x, asset_id_to_index, all_data.usd_oracles))
    for x in all_data.asset_pairs_xyk:
        edges.append(Edge(x, asset_id_to_index, all_data.usd_oracles))

    return edges, asset_id_to_index, all_tokens, index_to_asset_id


@dataclass
class BFSolution:
    outcomes: list[float]
    paths: list[list[int | None]]
    lambdas: list[float]
    deltas: list[float]
    routes: list[SingleInputAssetCvmRoute]


def route(
    input: Input,
    all_data: AllData,
    ctx: Ctx = Ctx(),
):
    """
    Bellman Ford inspired solution.
    The function divides the transaction if several paths (`splits``) and for each path
    find an optimal path without any modification.
    """
    # If max_depth or splits are not lists, convert them to lists
    max_depth = ctx.max_depth_of_route
    splits = ctx.forced_split_count
    if isinstance(max_depth, int):
        max_depth = [max_depth]
    if isinstance(splits, int):
        splits = [splits]

    # Create the list of edges and tokens

    venues, asset_id_to_index, all_tokens, index_to_asset_id = data2bf(all_data)

    # Number of tokens
    n = len(all_tokens)

    # Initialize the variables
    deltas: list[float] = [0] * len(venues)
    lambdas: list[float] = [0] * len(venues)
    paths: list[list[int | None]] = []
    outcomes: list[float] = [0]
    total_splits = sum(splits)

    # First and last nodes
    tendered_asset_index = asset_id_to_index[input.in_asset_id]
    received_asset_index = asset_id_to_index[input.out_asset_id]

    state = State()
    state.received_asset_index = received_asset_index
    state.edges = venues
    state.n = n

    # The dist and previous edge of each node for each length of the path
    state.reset_distances(max_depth)

    # For each max_depth and splits
    for max_depth_i, splits_i in zip(max_depth, splits):
        for _ in range(splits_i):  # The split variable is not used but left for clarity
            # Reset the dist and previous edge of each node for each length of the path
            state.reset_distances(max_depth)

            # Initialize the first node
            assert input.in_asset_amount > 0
            state.distances[tendered_asset_index][0] = Previous.start(input.in_asset_amount / total_splits)

            # Actualize the state
            state.depth = 0
            state.max_depth = max_depth_i

            for current_depth in range(max_depth_i):
                for venue_index, venue in enumerate(venues):
                    for current_asset_index in venue.nodes:
                        # if asset_index == received_asset_index:
                        #     raise Exception(state.distances[asset_index][current_depth].amount)
                        current_amount = state.distances[current_asset_index][current_depth].amount
                        if current_amount > 0:
                            maybe_better_venue = copy.deepcopy(venue)
                            next_asset_index = maybe_better_venue.other(current_asset_index)
                            # use the same edge if it has been used before
                            previous_asset_index = current_asset_index
                            # Go back in the path to check if the edge has been used before
                            for step_back in range(current_depth, 0, -1):
                                used_venue_index = state.distances[previous_asset_index][step_back].used_venue_index
                                previous_asset_index = venues[used_venue_index].other(previous_asset_index)
                                if used_venue_index == venue_index:
                                    maybe_better_venue.trade(
                                        previous_asset_index,
                                        state.distances[previous_asset_index][(step_back - 1)].amount,
                                    )
                            # Get the amount of the other token
                            maybe_better_received_amount = maybe_better_venue.trade(current_asset_index, current_amount)
                            if maybe_better_received_amount == 0:
                                raise Exception(maybe_better_venue)
                            logger.error(
                                f"{'=' * current_depth} for {current_amount}/{index_to_asset_id[current_asset_index]}->{maybe_better_received_amount}/{index_to_asset_id[next_asset_index]}"
                            )
                            # Update the amount of the other token if it is greater than the previous amount
                            best_known_received_amount = state.distances[next_asset_index][(current_depth + 1)].amount
                            if maybe_better_received_amount > best_known_received_amount:
                                state.distances[next_asset_index][(current_depth + 1)] = Previous(
                                    venue_index, maybe_better_received_amount
                                )

            # Get the optimal path

            received_amount = state.distances[received_asset_index][state.depth].amount
            # raise Exception(f"{received_amount} {state.distances[received_asset_index]}")
            for current_depth in range(1, max_depth_i + 1):
                maybe_received_amount = state.distances[received_asset_index][current_depth].amount
                if (
                    state.distances[received_asset_index][current_depth]
                    and maybe_received_amount > state.distances[received_asset_index][state.depth].amount
                    and maybe_received_amount > received_amount * ctx.loop_risk_ratio
                ):
                    state.depth = current_depth
                    received_amount = maybe_received_amount

            if received_amount == 0:
                raise Infeasible("No path retaining some value found")
            if state.depth == 0:  # if there is no path
                raise Infeasible("No path found")

            path: list[int | None] = [None] * state.depth

            # Rebuild the path
            next_asset_index = received_asset_index
            for current_depth in range(state.depth, 0, -1):
                logger.error(f"{next_asset_index}:{current_depth}:{state.distances[next_asset_index]}")
                used_venue_index = state.distances[next_asset_index][current_depth].used_venue_index
                assert used_venue_index is not None
                path[current_depth - 1] = used_venue_index
                next_asset_index = venues[used_venue_index].other(next_asset_index)

            # Use the path and update the edges
            tendered_amount = input.in_asset_amount / total_splits
            current_asset_index = asset_id_to_index[input.in_asset_id]
            for i in range(len(path)):
                venue = venues[path[i]]
                deltas[path[i]] += tendered_amount
                received_amount = venue.trade(current_asset_index, tendered_amount)
                lambdas[path[i]] += received_amount
                tendered_amount = received_amount
                current_asset_index = venue.other(current_asset_index)

            # Update the paths and outcomes
            assert tendered_amount > 0
            paths.append(path)
            outcomes.append(outcomes[-1] + tendered_amount)

    def build_next(parent, path):
        if len(path) > 0:
            head, rest = path[0], path[1:]
            venue = venues[head]
            received_amount = venue.venue.trade(parent.out_asset_id, parent.out_asset_amount)
            received_asset_id = venue.venue.other(parent.out_asset_id)

            logger.error(f"{parent.out_asset_amount}/{parent.out_asset_id}")
            logger.error(f"{received_amount}/{received_asset_id}")
            step = None
            if isinstance(venue.venue, AssetTransfers):
                step = Spawn(
                    in_asset_id=parent.out_asset_id,
                    out_asset_id=received_asset_id,
                    in_asset_amount=parent.out_asset_amount,
                    out_asset_amount=int(math.floor(received_amount)),
                    next=[],
                )
            else:
                step = Exchange(
                    in_asset_id=parent.out_asset_id,
                    out_asset_id=received_asset_id,
                    in_asset_amount=parent.out_asset_amount,
                    out_asset_amount=int(math.floor(received_amount)),
                    pool_id=str(venue.venue.pool_id),
                    next=[],
                )
            parent.next = [step]
            build_next(step, rest)

    route = SingleInputAssetCvmRoute(
        out_asset_id=input.in_asset_id,
        out_asset_amount=int(math.floor(input.in_asset_amount)),
        in_asset_id=input.in_asset_id,
        in_asset_amount=int(math.floor(input.in_asset_amount)),
        next=[],
    )
    build_next(route, sorted([path for path in paths], key=len)[0])
    _solution = BFSolution(outcomes, paths, lambdas, deltas, routes=[route])
    return [route]
