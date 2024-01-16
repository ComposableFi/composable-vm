# solves using convex optimization
import numpy as np
import cvxpy as cp
from strictly_typed_pandas import DataSet

MAX_RESERVE = 1e10

from simulation.routers.data import Input, PydanticDataSet, TAssetId, TNetworkId, AssetTransfers, AssetPairsXyk, AllData, new_data, new_input, new_pair


# clarabel cvxpy local mip
import itertools
import numpy as np

from  simulation.routers.dzmitry import solve, route

# simulate denom paths to and from chains, with center node
def populate_chain_dict(chains: dict[TNetworkId, list[TAssetId]], center_node: TNetworkId):
    # Add tokens with denom to Center Nod
    # Basic IBC transfer
    for chain, tokens in chains.items():
        if chain != center_node:
            chains[center_node].extend(f"{chain}/{token}" for token in tokens)

    1# Add tokens from Center Node to outers
    
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
    pair = new_pair(1, 1, 2, 0, 0, 1, 1, 100, 1_000_000, 1_000_000)
    data = new_data([pair], [])
    
    print(data)
    

def simulate():
    input = new_input("WETH", "ATOM", 2000, 1)
    CENTER_NODE, chains = simulate_all_to_all_connected_chains(input)
    print(chains)
    
    all_data = simulate_all_connected_pools(CENTER_NODE, chains) 
    print(all_data)
    
    print("=============== solving ========================")
    return route(input, all_data, all_cfmms, reserves, fees, cfmm_tx_cost, ibc_pools, input_amount)

def simulate_all_connected_pools(CENTER_NODE, chains):
    pools : list[AssetPairsXyk] = []
    transfers : list[AssetTransfers] = []
    
    # simulate in chain CFMMS
    all_token_pairs = []
    for _other_chain, other_tokens in chains.items():
        all_token_pairs.extend(itertools.combinations(other_tokens, 2))

    # simulate reserves and gas costs to CFMMS
    for i, pair in enumerate(all_token_pairs):
        [a, b] = np.random.uniform(95000000, 100510000, 2)
        pair = new_pair(i, pair[0], pair[1], 2, 0, 0, 1, 1, 100, a, b)

    # simulate crosschain transfers as "pools"
    for token_on_center in chains[CENTER_NODE]:
        for other_chain, other_tokens in chains.items():
            if other_chain != CENTER_NODE:
                for other_token in other_tokens:
                    # Check wether the chain has the token in center, or the other way around
                    # Could cause problems if chainName == tokensName (for example OSMOSIS)
                    if other_token in token_on_center or token_on_center in other_token:
                        all_cfmms.append((token_on_center, other_token))
                        reserves.append(np.random.uniform(10000, 11000, 2))
                        cfmm_tx_cost.append(np.random.uniform(0, 20))
                        ibc_pools += 1

    # simulate random fees
    fees.extend(np.random.uniform(0.97, 0.999) for _ in range(len(all_cfmms)))

def simulate_all_to_all_connected_chains(input: Input):
    CENTER_NODE = "CENTAURI"  # Name of center Node

    chains: dict[str, list[str]] = {
        "ETHEREUM": [input.in_token_id, "USDC", "SHIBA"],
        CENTER_NODE: [],
        "OSMOSIS": [input.out_token_id,"SCRT"],
    }
    populate_chain_dict(chains,CENTER_NODE)
    return CENTER_NODE,chains
    
if __name__ == "__main__":
    simulate()