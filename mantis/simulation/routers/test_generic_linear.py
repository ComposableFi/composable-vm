# solves using convex optimization
import math
import numpy as np
from simulation.routers.angeris_cvxpy import cvxpy_to_data

MAX_RESERVE = 1e10

from simulation.routers.data import (
    Ctx,
    Input,
    TId,
    TNetworkId,
    AssetTransfers,
    AssetPairsXyk,
    AllData,
    new_data,
    new_input,
    new_pair,
    new_transfer,
)


# clarabel cvxpy local mip
import itertools

from simulation.routers.generic_linear import route


# simulate denom paths to and from chains, with center node
def populate_chain_dict(chains: dict[TNetworkId, list[TId]], center_node: TNetworkId):
    # Add tokens with denom to Center Nod
    # Basic IBC transfer
    for chain, tokens in chains.items():
        if chain != center_node:
            chains[center_node].extend(f"{chain}/{token}" for token in tokens)

    1  # Add tokens from Center Node to outers

    # Simulate IBC transfer through Composable Cosmos
    for chain, tokens in chains.items():
        if chain != center_node:
            chains[chain].extend(
                f"{center_node}/{token}"
                for token in chains[center_node]
                if f"{chain}/" not in token
            )


def test_single_chain_single_cffm_route_full_symmetry_exist():
    input = new_input(1, 2, 100, 50)
    pair = new_pair(1, 1, 2, 0, 0, 1, 1, 1, 1_000_000, 1_000_000)
    data = new_data([pair], [])
    result = route(input, data)
    assert result[0] < 100
    assert result[0] > 95


def test_diamond():
    t1 = new_transfer("CENTAURI/ETHEREUM/USDC", "ETHEREUM/USDC", 10000, 100_000, 100_000, 0)
    t2 = new_transfer(
        "CENTAURI/ETHEREUM/USDC",
        "OSMOSIS/CENTAURI/ETHEREUM/USDC",
        1,
        100_000,
        100_000,
        0,
    )
    t3 = new_transfer("OSMOSIS/ETHEREUM/USDC", "ETHEREUM/USDC", 1, 100_000, 100_000, 0)

    s1 = new_pair(
        1, "ETHEREUM/USDC", "ETHEREUM/USDT", 0, 0, 1, 1, 200_000, 10_000, 10_000
    )
    s2 = new_pair(
        1,
        "OSMOSIS/ETHEREUM/USDC",
        "OSMOSIS/ETHEREUM/USDT",
        0,
        0,
        1,
        1,
        200_000,
        10_000,
        10_000,
    )
    s3 = new_pair(
        1,
        "CENTAURI/ETHEREUM/USDC",
        "CENTAURI/ETHEREUM/USDT",
        0,
        0,
        1,
        1,
        200_000,
        10_000,
        10_000,
    )
    s4 = new_pair(
        1,
        "OSMOSIS/CENTAURI/ETHEREUM/USDC",
        "OSMOSIS/ETHEREUM/USDC",
        0,
        0,
        1,
        1,
        200_000,
        10_000,
        10_000,
    )

    data = new_data([s1, s2, s3, s4], [t1, t2, t3])
    ctx = Ctx()
    input = new_input("CENTAURI/ETHEREUM/USDC", "ETHEREUM/USDC", 1_000, 50)
    result = route(
        input, data
    )
    solution = cvxpy_to_data(input, data, ctx, result)
    assert solution.children[0].name == "ETHEREUM/USDC"
    assert result.received(data.index_of_token("ETHEREUM/USDC")) == 1000.0000000000582
    raise NotImplementedError()

    # here we shutdown direct Centauri <-> Ethereum route, and force Centauri -> Osmosis -> Ethereum
    t1 = new_transfer(
        "CENTAURI/ETHEREUM/USDC", "ETHEREUM/USDC", 1_000_000, 100_000, 100_000, 0
    )
    data = new_data([s1, s2, s3, s4], [t1, t2, t3])
    input = new_input("CENTAURI/ETHEREUM/USDC", "ETHEREUM/USDC", 1_000, 50)
    result = route(
        input , data, ctx,
    )
    solution = cvxpy_to_data(input, data, ctx, result)
    assert math.floor(result[0]) == 909


def _test_big_numeric_range():
    input = new_input(1, 2, 100, 50)
    pair = new_pair(1, 1, 2, 0, 0, 1, 10, 1000, 10_000_000_000, 1_000_000_000)
    data = new_data([pair], [])
    result = route(input, data)
    print(result)


def test_simulate_all_connected_venues():
    np.random.seed(0)
    input = new_input("WETH", "ATOM", 2000, 1)
    CENTER_NODE, chains = simulate_all_to_all_connected_chains_topology(input)
    data = simulate_all_connected_venues(CENTER_NODE, chains)
    print(data)

    print("=============== solving ========================")
    ctx = Ctx()
    result = route(input, data, ctx)
    solution = cvxpy_to_data(input, data, ctx, result)
    print(result)


def simulate_all_connected_venues(CENTER_NODE, chains) -> AllData:
    pools: list[AssetPairsXyk] = []
    transfers: list[AssetTransfers] = []

    # simulate in chain CFMMS
    all_token_pairs = []
    for _other_chain, other_tokens in chains.items():
        all_token_pairs.extend(itertools.combinations(other_tokens, 2))

    # simulate reserves and gas costs to CFMM
    for i, x in enumerate(all_token_pairs):
        [a, b] = np.random.randint(9500, 10500, 2)
        fee = np.random.randint(0, 10_000)
        x = new_pair(i, x[0], x[1], fee, fee, 1, 1, 1_000, a, b)
        pools.append(x)

    # simulate crosschain transfers as "pools"
    all_token_transfers = []
    for token_on_center in chains[CENTER_NODE]:
        for other_chain, other_tokens in chains.items():
            if other_chain != CENTER_NODE:
                for other_token in other_tokens:
                    # Check wether the chain has the token in center, or the other way around
                    # Could cause problems if chainName == tokensName (for example OSMOSIS)
                    if other_token in token_on_center or token_on_center in other_token:
                        all_token_transfers.append((token_on_center, other_token))

    for _i, x in enumerate(all_token_transfers):
        abc = np.random.randint(9500, 10500, 2)
        a = abc[0]
        b = abc[1]
        tx_cost = np.random.randint(0, 1_00)
        fee = np.random.randint(0, 10_000)
        x = new_transfer(x[0], x[1], tx_cost, a, b, fee)
        transfers.append(x)

    return new_data(pools, transfers)


def simulate_all_to_all_connected_chains_topology(input: Input):
    CENTER_NODE = "CENTAURI"  # Name of center Node

    chains: dict[str, list[str]] = {
        "ETHEREUM": [input.in_token_id, "USDC", "SHIBA"],
        CENTER_NODE: [],
        "OSMOSIS": [input.out_token_id, "SCRT"],
    }
    populate_chain_dict(chains, CENTER_NODE)
    return CENTER_NODE, chains


if __name__ == "__main__":
    _test_simulate_all_connected_venues()
