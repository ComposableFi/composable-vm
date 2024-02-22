import copy
import os
import threading as th

# import time
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
    # In the future it could be used in hipergraph algorithms with a few changes

    U: list[int]  # nodes of the edge
    B: list[TAmount]  # amount of each token in the edge
    W: list[int]  # weight of each token in the edge
    F: list[float]  # fee of each token in the edge
    CF: list[float]  # constant fee of each token in the edge

    def toFloatOrZero(self, x):  # Cast to float using 0.0 when it fails
        return float(x) if x else 0.0

    def __init__(
        self,
        e: [AssetTransfers | AssetPairsXyk],
        tokensIds: dict[TId, int],
        usd_oracles: dict[TId, int],
    ):
        # Creates an edge in base to a pool
        if isinstance(e, AssetTransfers):
            self.__initFromTransfers(e, tokensIds, usd_oracles)
        else:
            self.__initFromPairsXyk(e, tokensIds, usd_oracles)

    def __initFromTransfers(
        self, e: AssetTransfers, tokensIds: dict[TId, int], usd_oracles: dict[TId, int]
    ):
        self.U = [tokensIds[e.in_asset_id], tokensIds[e.out_asset_id]]
        self.B = [e.amount_of_in_token, e.amount_of_out_token]
        self.W = [1, 1]
        self.F = [
            float(e.fee_per_million) / 1_000_000.0,
            float(e.fee_per_million) / 1_000_000.0,
        ]
        self.CF = [0, 0]

    def __initFromPairsXyk(
        self, e: AssetPairsXyk, tokensIds: dict[TId, int], usd_oracles: dict[TId, int]
    ):
        self.U = [tokensIds[e.in_asset_id], tokensIds[e.out_asset_id]]
        self.B = [e.in_token_amount, e.out_token_amount]
        self.W = [e.weight_of_a, e.weight_of_b]
        self.F = [self.toFloatOrZero(e.fee_in), self.toFloatOrZero(e.fee_out)]
        self.CF = [0, 0]

    def GetAmount(self, Ti, Xi):
        # Send Xi amount of token Ti and return the amount of the other token
        i, o = 0, 1
        if Ti == self.U[1]:
            i, o = 1, 0
        Xi = (Xi - self.CF[i]) * (1 - self.F[i])
        return self.B[o] * (
            1 - (self.B[i] / (self.B[i] + Xi)) ** (self.W[i] / self.W[o])
        )

    def DoChange(self, Ti, Xi):
        # Actually do the change of the amount of the tokens
        i, o = 0, 1
        if Ti == self.U[1]:
            i, o = 1, 0
        Xi = (Xi - self.CF[i]) * (1 - self.F[i])
        result = self.B[o] * (
            1 - (self.B[i] / (self.B[i] + Xi)) ** (self.W[i] / self.W[o])
        )
        self.B[i] += Xi
        self.B[o] -= result
        return result

    def GetOther(self, Ti):  # Asumes only 2 nodes in the edge
        if Ti == self.U[0]:
            return self.U[1]
        return self.U[0]

    def __repr__(self):
        return f"Edge({self.U}, {self.B}, {self.W}, {self.F}, {self.CF})"


class State:
    # A class that represent the state of the algorithm
    # It's used to pass the state to the threads if executed in parallel
    max_depth: int
    depth: int
    dist: list[list[tuple[int, float]]]
    dlock: list[th.Lock]
    u_end: int
    edges: list[Edge]
    revision: bool
    Nopts: int
    j: int
    n: int

    def __init__(self):
        self.dist = None
        self.max_depth = None
        self.depth = None
        self.u_end = None
        self.edges = None
        self.revision = None
        self.dlock = None
        self.Nopts = None
        self.j = None
        self.n = None


def Range(e0, e1, state):
    # A function that is used by the threads to process a range of edges

    edges = state.edges
    j = state.j

    # For each edge in the range run the BF step
    dist = state.dist
    for ei in range(e0, e1):
        e = edges[ei]
        for u in e.U:
            if dist[j * state.n + u][1] == 0:
                continue
            v = e.GetOther(u)
            if state.revision:
                ee = copy.deepcopy(e)
                vv = u
                for jj in range(j, 0, -1):
                    pad = dist[jj * state.n + vv][0]
                    vv = edges[pad].GetOther(vv)
                    if pad == ei:
                        ee.DoChange(vv, dist[(jj - 1) * state.n + vv][1])
            else:
                ee = e
            Xv = ee.GetAmount(u, dist[j * state.n + u][1])
            state.dlock[v].acquire()  # Lock the node
            if dist[(j + 1) * state.n + v][1] < Xv:
                dist[(j + 1) * state.n + v] = (ei, Xv)
            state.dlock[v].release()  # Unlock the node


# Bellman Ford based solution

# The function divides the transaction if several paths (splits) and for each path
# find an optimal path using the Bellman Ford algorithm without any modification.
#
# If the revision parameter is True, in each step the edge will be used with the information
# of the path that reached the first node. This might be important in loops.
#
# The parameters of the functions allows to go over the runtime-accuracy tradeoff


def conversor(
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
    splits: int = 1000,  # The number of flow units in which the amount is divided
    revision=True,  # When uses an edge, check if the edge has been used before and if so, use the same edge
    Nproces=None,  # A parameter used for paralell programing. For now, it seems to be best to use only one threat than paralell programming
):
    # If the number of processes is not given, use the number of cpus
    if Nproces is None:
        Nproces = os.cpu_count()

    # If max_depth or splits are not lists, convert them to lists
    if isinstance(max_depth, int):
        max_depth = [max_depth]
    if isinstance(splits, int):
        splits = [splits]

    # Create the list of edges and tokens

    edges, tokensIds, all_tokens = conversor(all_data)

    # Number of tokens
    n = len(all_tokens)

    # Initialize the variables
    deltas: list[float] = [0] * len(edges)
    lambdas: list[float] = [0] * len(edges)
    paths: list[list[int]] = []
    outcomes: list[float] = [0]
    totSplits = sum(splits)

    # First and last nodes
    u_init = tokensIds[input.in_token_id]
    u_end = tokensIds[input.out_token_id]

    # The state of the algorithm that can be passed to the threads if executed in parallel
    state = State()
    state.u_end = u_end
    state.edges = edges
    state.revision = revision
    state.dlock = [th.Lock() for i in range(n)]
    state.n = n

    # If the number of processes is greater than 1, divide the edges into Nproces parts
    if Nproces > 1:
        e0 = [i * len(edges) // Nproces for i in range(Nproces)]
        e1 = [(i + 1) * len(edges) // Nproces for i in range(Nproces)]
        e1[-1] = len(edges)
        state.dlock = [th.Lock() for i in range(n)]

    # The dist and previous edge of each node for each length of the path
    state.dist = [(None, 0)] * ((max(max_depth) + 1) * n)

    # For each max_depth and splits
    for max_depth_i, splits_i in zip(max_depth, splits):
        for split in range(
            splits_i
        ):  # The split variable is not used but left for clarity
            # Reset the dist and previous edge of each node for each length of the path
            for i in range(((max(max_depth) + 1) * n)):
                state.dist[i] = (None, 0)

            # Initialize the first node
            state.dist[u_init] = (None, input.in_amount / (totSplits))

            # Actualizate the state
            state.depth = 0
            state.max_depth = max_depth_i

            # Process each legth of the path
            for step in range(max_depth_i):
                #    start = time.time()
                if (
                    Nproces > 1
                ):  # If the number of processes is greater than 1, use the threads
                    state.j = step
                    threads = [
                        th.Thread(target=Range, args=(e0[i], e1[i], state))
                        for i in range(Nproces)
                    ]
                    for t in threads:
                        t.start()
                    for t in threads:
                        t.join()
                else:  # If the number of processes is 1, use the main thread
                    for ei, e in enumerate(edges):
                        for u in e.U:
                            if state.dist[step * state.n + u][1] == 0:
                                continue
                            v = e.GetOther(u)
                            if state.revision:  # If the revision is active, use the same edge if it has been used before
                                ee = copy.deepcopy(e)
                                vv = u
                                # Go back in the path to check if the edge has been used before
                                for jj in range(step, 0, -1):
                                    pad = state.dist[jj * state.n + vv][0]
                                    vv = edges[pad].GetOther(vv)
                                    if pad == ei:
                                        ee.DoChange(
                                            vv, state.dist[(jj - 1) * state.n + vv][1]
                                        )
                            else:
                                ee = e  # If the revision is not active, use the edge
                            # Get the amount of the other token
                            Xv = ee.GetAmount(u, state.dist[step * state.n + u][1])
                            # Update the amount of the other token if it is greater than the previous amount
                            if state.dist[(step + 1) * state.n + v][1] < Xv:
                                state.dist[(step + 1) * state.n + v] = (ei, Xv)

            # Get the optimal path
            for j in range(1, max_depth_i + 1):
                if state.dist[j * n + u_end] and (
                    state.depth == 0
                    or state.dist[j * n + u_end][1]
                    > state.dist[state.depth * n + u_end][1]
                ):
                    state.depth = j

            if state.depth == 0:  # if there is no path
                raise Exception("No path found")

            path: list[int] = [0] * state.depth
            v = u_end

            # Rebuild the path
            for j in range(state.depth, 0, -1):
                path[j - 1] = state.dist[j * n + v][0]
                v = edges[path[j - 1]].GetOther(v)

            # Use the path and update the edges
            Xi = input.in_amount / (totSplits)
            u = tokensIds[input.in_token_id]
            for i in range(len(path)):
                e = edges[path[i]]
                deltas[path[i]] += Xi
                Xj = e.DoChange(u, Xi)
                lambdas[path[i]] += Xj
                Xi = Xj
                u = e.GetOther(u)

            # Update the paths and outcomes
            assert Xi > 0
            paths.append(path)
            outcomes.append(outcomes[-1] + Xi)

    return outcomes[-1], outcomes[-2], paths, lambdas, deltas


# A way to fix some parameters of the function, use for a functional approach
def BuildRoute(max_depth, splits, revision, Nproces=1):
    def _route(input: Input, all_data: AllData, _ctx: Ctx = Ctx()):
        return route(input, all_data, _ctx, max_depth, splits, revision, Nproces)

    return _route
