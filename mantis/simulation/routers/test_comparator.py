# solves using convex optimization
import numpy as np
import time

MAX_RESERVE = 1e10

from simulation.routers.data import (
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
import subprocess

from simulation.routers import lautaro, dzmitry
from simulation.routers.data import Ctx

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




def _test_simulate_all_connected_venues(routers):
    input = new_input("WETH", "ATOM", 2000, 1)
    CENTER_NODE, chains = simulate_all_to_all_connected_chains_topology(input)
    print(chains)

    all_data = simulate_all_connected_venue(CENTER_NODE, chains)
    print(all_data)
    print(all_data.all_tokens)
    print(all_data.index_of_token("WETH"))
    print(all_data.index_of_token("ATOM"))

    print("=============== solving ========================")
    
    results = []
    values = {}
    for (name, route) in routers.items():
        start = time.time()
        result = route(input, all_data)
        end = time.time()
        results.append(f'{name} result: {result} in time: {end - start}')
        values[name] = (result, end - start)
    for result in results:
        print(result)
    return values

def simulate_all_connected_venue(CENTER_NODE, chains) -> AllData:
    pools: list[AssetPairsXyk] = []
    transfers: list[AssetTransfers] = []

    # simulate in chain CFMMS
    all_token_pairs = []
    for _other_chain, other_tokens in chains.items():
        all_token_pairs.extend(itertools.combinations(other_tokens, 2))

    # simulate reserves and gas costs to CFMMS
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
        tx_cost = np.random.randint(0, 1_000)
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
    _test_simulate_all_connected_venues({
        "Lautaro A0": lautaro.BuildRoute(5,1000,False),
        "Lautaro A": lautaro.BuildRoute(5,1000,True),
        "Lautaro B": lautaro.BuildRoute(10,1000,True),
        "Lautaro C": lautaro.BuildRoute(15,1000,True),
        "Dzmitry": dzmitry.route,
    })
