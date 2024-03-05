import copy
from dataclasses import dataclass
import os
from simulation.routers.errors import Infeasible

from simulation.routers.data import (
    AllData,
    AssetPairsXyk,
    AssetTransfers,
    Ctx,
    Input,
    TAmount,
    TId,
)


class Edge:
    # A class that represent an edge in an useful way
    # In the future it could be used in hypergraph algorithms with a few changes

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

    def __initFromTransfers(self, e: AssetTransfers, tokensIds: dict[TId, int], usd_oracles: dict[TId, int]):
        self.nodes = [tokensIds[e.in_asset_id], tokensIds[e.out_asset_id]]
        # raise Exception("transfer amounts are not subject to same logic as exchanges")
        self.balances = [e.in_token_amount, e.out_token_amount]
        self.weights = [1, 1]
        self.fees = [
            float(e.fee_per_million) / 1_000_000.0,
            float(e.fee_per_million) / 1_000_000.0,
        ]
        self.constant_fees = [0, 0]

    def __initFromPairsXyk(self, e: AssetPairsXyk, tokensIds: dict[TId, int], usd_oracles: dict[TId, int]):
        self.nodes = [tokensIds[e.in_asset_id], tokensIds[e.out_asset_id]]
        self.balances = [e.in_token_amount, e.out_token_amount]
        self.weights = [e.weight_a, e.weight_b]
        self.fees = [self.toFloatOrZero(e.fee_in), self.toFloatOrZero(e.fee_out)]
        self.constant_fees = [0, 0]


    def trade(self, Ti, Xi):
        # Actually do the change of the amount of the tokens
        i, o = 0, 1
        if Ti == self.nodes[1]:
            i, o = 1, 0
        Xi = (Xi - self.constant_fees[i]) * (1 - self.fees[i])
        result = self.balances[o] * (1 - (self.balances[i] / (self.balances[i] + Xi)) ** (self.weights[i] / self.weights[o]))
        self.balances[i] += Xi
        self.balances[o] -= result
        return result

    def GetOther(self, Ti):  # Assumes only 2 nodes in the edge
        if Ti == self.nodes[0]:
            return self.nodes[1]
        return self.nodes[0]

    def __repr__(self):
        return f"Edge({self.nodes}, {self.balances}, {self.weights}, {self.fees}, {self.constant_fees})"


@dataclass
class Previous:
    used_venue_index: int | None
    """
    Used venue index to arrive to `amount`
    """
    amount: int
    
    def default() -> 'Previous':
        return Previous(None, 0)
    
    def start(tendered_asset) -> 'Previous':
        return Previous(None, tendered_asset) 
    
class State:
    # A class that represent the state of the algorithm
    # It's used to pass the state to the threads if executed in parallel
    max_depth: int
    depth: int
    distance: list[Previous]
    received_asset_index: int
    edges: list[Edge]
    revision: bool
    j: int
    n: int

    def __init__(self):
        self.distance = None
        self.max_depth = None
        self.depth = None
        self.received_asset_index = None
        self.edges = None
        self.revision = None
        self.j = None
        self.n = None

# Bellman Ford based solution
# The function divides the transaction if several paths (splits) and for each path
# find an optimal path using the Bellman Ford algorithm without any modification.
#
# If the revision parameter is True, in each step the edge will be used with the information
# of the path that reached the first node. This might be important in loops.
#
# The parameters of the functions allows to go over the runtime-accuracy tradeoff
def data2bf(
    all_data: AllData,
):
    edges: list[Edge] = []
    all_tokens = all_data.all_tokens
    tokensIds = {x: i for i, x in enumerate(all_tokens)}
    for x in all_data.asset_transfers:
        edges.append(Edge(x, tokensIds, all_data.usd_oracles))
    for x in all_data.asset_pairs_xyk:
        edges.append(Edge(x, tokensIds, all_data.usd_oracles))

    return edges, tokensIds, all_tokens

def route(
    input: Input,
    all_data: AllData,
    _ctx: Ctx = Ctx(),  # Context
    max_depth: int = 5,  # The maximum number of edges that can be used
    splits: int = 2,  # The number of flow units in which the amount is divided
    revision=True,  # When uses an edge, check if the edge has been used before and if so, use the same edge
):
    # If max_depth or splits are not lists, convert them to lists
    if isinstance(max_depth, int):
        max_depth = [max_depth]
    if isinstance(splits, int):
        splits = [splits]

    # Create the list of edges and tokens

    venues, asset_id_to_index, all_tokens = data2bf(all_data)

    # Number of tokens
    n = len(all_tokens)

    # Initialize the variables
    deltas: list[float] = [0] * len(venues)
    lambdas: list[float] = [0] * len(venues)
    paths: list[list[int]] = []
    outcomes: list[float] = [0]
    total_splits = sum(splits)

    # First and last nodes
    tendered_asset_index = asset_id_to_index[input.in_token_id]
    received_asset_index = asset_id_to_index[input.out_token_id]

    state = State()
    state.received_asset_index = received_asset_index
    state.edges = venues
    state.revision = revision
    state.n = n

    # The dist and previous edge of each node for each length of the path
    state.distance = [Previous.default()] * ((max(max_depth) + 1) * n)

    # For each max_depth and splits
    for max_depth_i, splits_i in zip(max_depth, splits):
        for _ in range(splits_i):  # The split variable is not used but left for clarity
            # Reset the dist and previous edge of each node for each length of the path
            for i in range(((max(max_depth) + 1) * n)):
                state.distance[i] = Previous.default()

            # Initialize the first node
            state.distance[tendered_asset_index] = Previous.start(input.in_amount / total_splits)

            # Actualize the state
            state.depth = 0
            state.max_depth = max_depth_i

            # Process each length of the path
            for step in range(max_depth_i):
                    for venue_index, venue in enumerate(venues):
                        for asset_index in venue.nodes:
                            if state.distance[step * state.n + asset_index].amount > 0:
                                other_node_index = venue.GetOther(asset_index)
                                maybe_better_venue = copy.deepcopy(venue)
                                if (
                                    state.revision
                                ):  # If the revision is active, use the same edge if it has been used before
                                    previous_asset_index = asset_index
                                    # Go back in the path to check if the edge has been used before
                                    for step_back in range(step, 0, -1):
                                        used_venue_index = state.distance[step_back * state.n + previous_asset_index].used_venue_index
                                        previous_asset_index = venues[used_venue_index].GetOther(previous_asset_index)
                                        if used_venue_index == venue_index:
                                            maybe_better_venue.trade(previous_asset_index, state.distance[(step_back - 1) * state.n + previous_asset_index].amount)
                                # Get the amount of the other token
                                received_amount = maybe_better_venue.trade(asset_index, state.distance[step * state.n + asset_index].amount)
                                # Update the amount of the other token if it is greater than the previous amount
                                if state.distance[(step + 1) * state.n + other_node_index].amount < received_amount:
                                    state.distance[(step + 1) * state.n + other_node_index] = Previous(venue_index, received_amount)

            # Get the optimal path
            for current_depth in range(1, max_depth_i + 1):
                if state.distance[current_depth * n + received_asset_index] and (
                    state.depth == 0 or state.distance[current_depth * n + received_asset_index].amount > state.distance[state.depth * n + received_asset_index].amount
                ):
                    state.depth = current_depth

            if state.depth == 0:  # if there is no path
                raise Infeasible("No path found")

            path: list[int] = [0] * state.depth

            # Rebuild the path
            other_node_index = received_asset_index
            for current_depth in range(state.depth, 0, -1):
                path[current_depth - 1] = state.distance[current_depth * n + other_node_index].used_venue_index
                other_node_index = venues[path[current_depth - 1]].GetOther(other_node_index)

            # Use the path and update the edges
            tendered_amount = input.in_amount / total_splits
            asset_index = asset_id_to_index[input.in_token_id]
            for i in range(len(path)):
                venue = venues[path[i]]
                deltas[path[i]] += tendered_amount
                received_amount = venue.trade(asset_index, tendered_amount)
                lambdas[path[i]] += received_amount
                tendered_amount = received_amount
                asset_index = venue.GetOther(asset_index)

            # Update the paths and outcomes
            assert tendered_amount > 0
            paths.append(path)
            outcomes.append(outcomes[-1] + tendered_amount)

    return outcomes[-1], outcomes[-2], paths, lambdas, deltas