# solves using convex optimization
import numpy as np
import cvxpy as cp

MAX_RESERVE = 1e10

from simulation.routers.data import Input, TAssetId, TNetworkId, AssetTransfers, AssetPairsXyk, AllData


# clarabel cvxpy local mip
import itertools
import numpy as np

from  simulation.routers.dzmitry import solve, route

# simulate denom paths to and from chains, with center node
def populate_chain_dict(chains: dict[TNetworkId, list[TAssetId]], center_node: TNetworkId):
    # Add tokens with denom to Center Node
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
    input = Input.max("1", "2", 100, 100)
    
    

def simulate():
    print("=============== chains and tokens ========================")
    CENTER_NODE = "CENTAURI"  # Name of center Node

    ORIGIN_TOKEN = "WETH"
    OBJ_TOKEN = "ATOM"

    chains: dict[str, list[str]] = {
        "ETHEREUM": ["WETH", "USDC", "SHIBA"],
        CENTER_NODE: [],
        "OSMOSIS": ["ATOM","SCRT"],
    }
    populate_chain_dict(chains,CENTER_NODE)

    all_tokens = []
    all_cfmms = []
    reserves = []
    fees = []
    cfmm_tx_cost = []
    ibc_pools = 0
    tol = 1e-4

    print(chains)

    # simulate in chain CFMMS
    for other_chain, other_tokens in chains.items():
        all_tokens.extend(other_tokens)
        all_cfmms.extend(itertools.combinations(other_tokens, 2))

    # simulate reserves and gas costs to CFMMS
    for cfmm in all_cfmms:
        reserves.append(np.random.uniform(95000000, 100510000, 2))
        cfmm_tx_cost.append(np.random.uniform(0, 20))

    # simulate IBC "pools"
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

    print(reserves)

    for i, token in enumerate(all_tokens):
        print(i, token)

    for i, cfmm in enumerate(all_cfmms):
        print(i, cfmm)
    input_amount = 2000
    

    print("=============== solving ========================")
    return route(ORIGIN_TOKEN, OBJ_TOKEN, all_tokens, all_cfmms, reserves, fees, cfmm_tx_cost, ibc_pools, input_amount)
    
if __name__ == "__main__":
    simulate()