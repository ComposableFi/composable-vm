import itertools
import numpy as np

from  mantis.simulation.routers.bruno import solve
from mantis.simulation.routers.data import TNetworkId, TAssetId

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
    _deltas, _lambdas, psi, n = solve(
        all_tokens, 
        all_cfmms, 
        reserves, 
        cfmm_tx_cost, 
        fees, 
        ibc_pools, 
        ORIGIN_TOKEN,
        input_amount,
        OBJ_TOKEN
        )

    to_look_n: list[float] = []
    for i in range(len(all_cfmms)):
        to_look_n.append(n[i].value)

    _max = 0
    for t in sorted(to_look_n):
        try:
            d2, l2, p2, n2 =  solve(
                all_tokens,
                all_cfmms,
                reserves,
                cfmm_tx_cost,
                fees,
                ibc_pools,
                ORIGIN_TOKEN,
                input_amount,
                OBJ_TOKEN,
                [1 if value <= t else 0 for value in to_look_n],
            )
            if psi.value[all_tokens.index(OBJ_TOKEN)] > _max:
                d_max, l_max, p_max, n_max = d2, l2, p2, n2 
            print("---")
        except:
            continue
    eta = n_max
    eta_change = True
    print("---------")
    lastp_value = psi.value[all_tokens.index(OBJ_TOKEN)]
    while eta_change:
        try:
            eta_change = False

            for idx, delta in enumerate(d_max):
                if all(delta_i.value < 1e-04 for delta_i in delta):
                    n_max[idx] = 0
                    eta_change = True
            d_max, l, psi, eta = solve(
                all_tokens,
                all_cfmms,
                reserves,
                cfmm_tx_cost,
                fees,
                ibc_pools,
                ORIGIN_TOKEN,
                input_amount,
                OBJ_TOKEN,
                eta,
            )

        except:
            continue

    print("---")
    deltas, lambdas, psi, eta = solve(
                    all_tokens,
                    all_cfmms,
                    reserves,
                    cfmm_tx_cost,
                    fees,
                    ibc_pools,
                    ORIGIN_TOKEN,
                    input_amount,
                    OBJ_TOKEN,
                    eta,
                )
    m = len(all_cfmms)
    for i in range(m):
        print(
            f"Market {all_cfmms[i][0]}<->{all_cfmms[i][1]}, delta: {deltas[i].value}, lambda: {lambdas[i].value}, eta: {eta[i].value}",
        )

    print(psi.value[all_tokens.index(OBJ_TOKEN)],lastp_value)
    # basically, we have matrix where rows are in tokens (DELTA)
    # columns are outs (LAMBDA)
    # so recursively going DELTA-LAMBDA and subtracting values from cells
    # will allow to build route with amounts 
    return (psi.value[all_tokens.index(OBJ_TOKEN)],lastp_value)
    
if __name__ == "__main__":
    simulate()