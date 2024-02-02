import itertools
import numpy as np
import cvxpy as cp
import pandas as pd
import random

from typing import TypeVar, Tuple, Dict, List
from tqdm import tqdm

TAssetId = TypeVar("TAssetId")
TNetworkId = TypeVar("TNetworkId")

# Creating a class to represent the CFMM
class CFMM:
    def __init__(
        self,
        tokens: Tuple[str, str],
        reserves: np.array,
        tx_cost: float,
        pool_fee: float,
        is_transfer: bool,
    ) -> None:
        self.tokens = tokens
        self.reserves = reserves
        self.pool_fee = pool_fee
        self.is_transfer = is_transfer
        self.tx_cost = tx_cost

    def __len__(self) -> int:
        return len(self.tokens)

    def __str__(self) -> str:
        return f"CFMM({self.tokens}, {self.reserves}, {self.tx_cost}, {self.pool_fee}, {self.is_transfer})"

    def __repr__(self) -> str:
        return str(self)
    
class Swap: 
    
    def __init__(self, in_token: str, in_token_amount: float, out_token: str, out_token_amount: float, swap_type: str) -> None:
        self.in_token = in_token
        self.in_token_amount = in_token_amount
        self.out_token = out_token
        self.out_token_amount = out_token_amount
        self.swap_type = swap_type
        
    def __str__(self) -> str:
        return f'Swap({self.in_token}, {self.in_token_amount}, {self.out_token}, {self.out_token_amount}, {self.swap_type})'
    
    def __repr__(self) -> str:
        return str(self)

# Container class for the simulation environment
class OrderRoutingSimulationEnvironment:
    def __init__(
        self, center_node: str, max_reserve: float, chains: Dict[str, List[str]]
    ):
        self.center_node = center_node
        self.max_reserve = max_reserve
        self.chains = chains

        # Populate the chain dictionary
        populate_chain_dict(self.chains, self.center_node)

        self.all_tokens = []
        self.all_cfmm_combinations = []
        self.all_cfmms = []

        for other_chain, other_tokens in chains.items():
            self.all_tokens.extend(other_tokens)
            self.all_cfmm_combinations.extend(itertools.combinations(other_tokens, 2))

        for cfmm in self.all_cfmm_combinations:
            random_reserves = np.random.uniform(9500, 10051, 2)
            random_tx_cost = np.random.uniform(0, 20)
            random_pool_fee = np.random.uniform(0.97, 0.99)
            mm = CFMM(cfmm, random_reserves, random_tx_cost, random_pool_fee, False)
            self.all_cfmms.append(mm)

        # Creating the CFMMs that are transfers
        for token_on_center in chains[self.center_node]:
            for other_chain, other_tokens in chains.items():
                if other_chain != self.center_node:
                    for other_token in other_tokens:
                        # Check wether the chain has the token in center, or the other way around
                        # Could cause problems if chainName == tokensName (for example OSMOSIS)
                        if (
                            other_token in token_on_center
                            or token_on_center in other_token
                        ):
                            random_reserves = np.random.uniform(9500, 10051, 2)
                            random_tx_cost = np.random.uniform(0, 20)
                            random_pool_fee = np.random.uniform(0.97, 0.99)
                            cfmm = CFMM(
                                (token_on_center, other_token),
                                random_reserves,
                                random_tx_cost,
                                random_pool_fee,
                                True,
                            )
                            self.all_cfmms.append(cfmm)

        self.ibc_pools = [cfmm for cfmm in self.all_cfmms if cfmm.is_transfer]
        self.non_ibc_pools = [cfmm for cfmm in self.all_cfmms if not cfmm.is_transfer]
        self.num_ibc_pools = len(self.ibc_pools)
        self.num_non_ibc_pools = len(self.non_ibc_pools)

        self.mapping_matrices = get_mapping_matrices(self.all_tokens, self.all_cfmms)


def flip(p: float) -> bool:
    return random.random() < p


def populate_chain_dict(
    chains: dict[TNetworkId, list[TAssetId]], center_node: TNetworkId
):
    # Add tokens with denom to Center Node
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


def get_mapping_matrices(all_tokens, all_cfmms):
    count_tokens = len(all_tokens)

    mapping_matrices = []
    for cfmm in all_cfmms:
        n_i = len(cfmm.tokens)
        A_i = np.zeros((count_tokens, n_i))
        for i, token in enumerate(cfmm.tokens):
            A_i[all_tokens.index(token), i] = 1
        mapping_matrices.append(A_i)

    return mapping_matrices

def get_tokens_from_trade(trade, all_tokens): 
    non_zero_indices = np.nonzero(trade)[0]
    traded_tokens = [all_tokens[i] for i in non_zero_indices]
    traded_amounts = [trade[i] for i in non_zero_indices]
    
    return dict(zip(traded_tokens, traded_amounts))


def solve_with_unknown_eta(
    origin_token: str,
    number_of_init_tokens: float,
    obj_token: str,
    all_tokens,
    all_cfmms,
    mapping_matrices,
    MAX_RESERVE=1e10,
):
    """In this function we solve the optimal routing problem with unknown eta values. These eta values go between 0 and 1.

    Args:
        origin_token (str): The origin token
        number_of_init_tokens (float): The number of tokens we start with
        obj_token (str): The token we want to end up with
        all_tokens (_type_): A list of all tokens in the simulation
        all_cfmms (_type_): A list of all possible CFMMs in the simulation
        mapping_matrices (_type_): Matrices which map the tokens in local indices for each CFMM to global indices
        MAX_RESERVE (_type_, optional): Max reserve of tokens in any CFMM. Defaults to 1e10.

    Returns:
        _type_: _description_
    """
    # Build local-global matrices
    count_tokens = len(all_tokens)
    current_assets = np.zeros(count_tokens)  # Initial assets
    current_assets[all_tokens.index(origin_token)] = number_of_init_tokens

    # Getting the cfmms which have been chosen and their mapping matrices
    all_reserves = [cfmm.reserves for cfmm in all_cfmms]
    all_fees = [cfmm.pool_fee for cfmm in all_cfmms]
    all_tx_cost = [cfmm.tx_cost for cfmm in all_cfmms]

    deltas = [cp.Variable(len(cfmm), nonneg=True) for cfmm in all_cfmms]
    lambdas = [cp.Variable(len(cfmm), nonneg=True) for cfmm in all_cfmms]

    eta = cp.Variable(len(all_cfmms), nonneg=True)

    psi = cp.sum(
        [
            A_i @ (LAMBDA - DELTA)
            for A_i, DELTA, LAMBDA in zip(mapping_matrices, deltas, lambdas)
        ]
    )

    # Creating the objective
    objective = cp.Maximize(
        psi[all_tokens.index(obj_token)]
        - cp.sum([eta[i] * all_tx_cost[i] for i in range(len(all_cfmms))])
    )

    # Creating the reserves
    new_reserves_list = [
        R + gamma * D - L
        for R, gamma, D, L in zip(all_reserves, all_fees, deltas, lambdas)
    ]

    # Creating constraints
    constraints = [psi + current_assets >= 0]

    # Pool constraints
    for i, cfmm in enumerate(all_cfmms):
        new_reserves = new_reserves_list[i]
        old_reserves = all_reserves[i]
        if cfmm.is_transfer:
            constraints.append(cp.sum(new_reserves) >= cp.sum(old_reserves))
            constraints.append(new_reserves >= 0)
        else:
            # In this case we don't have a transfer pool but a regular swap
            constraints.append(cp.geo_mean(new_reserves) >= cp.geo_mean(old_reserves))

    for i in range(len(all_cfmms)):
        constraints.append(deltas[i] <= MAX_RESERVE * eta[i])

    # Set up and solve the problem
    problem = cp.Problem(objective, constraints)

    problem.solve(verbose=True, solver="CLARABEL", qcp=False)

    return deltas, lambdas, psi, eta


def solve_with_known_eta(
    origin_token: str,
    number_of_init_tokens: float,
    obj_token: str,
    all_tokens,
    all_cfmms,
    mapping_matrices,
    eta,
    MAX_RESERVE=1e10,
):
    """In this function we solve the optimal routing problem with known eta values. These values are either 0 or 1.
    How we approach this is by not including any CFMMs into the constraints that have an
    eta value equal to 0. This way, we can solve the problem with CVXPY and not run into numerical issues.

    Args:
        origin_token (str): The origin token
        number_of_init_tokens (float): The number of tokens we start with
        obj_token (str): The token we want to end up with
        all_tokens (_type_): A list of all tokens in the simulation
        all_cfmms (_type_): A list of all possible CFMMs in the simulation
        mapping_matrices (_type_): Matrices which map the tokens in local indices for each CFMM to global indices
        eta (_type_): An array of boolean values indicating whether a CFMM is used or not
    """
    # Build local-global matrices
    count_tokens = len(all_tokens)
    current_assets = np.zeros(count_tokens)  # Initial assets
    current_assets[all_tokens.index(origin_token)] = number_of_init_tokens

    # Getting the cfmms which have been chosen and their mapping matrices
    all_reserves = [cfmm.reserves for cfmm in all_cfmms]
    all_fees = [cfmm.pool_fee for cfmm in all_cfmms]
    all_tx_cost = [cfmm.tx_cost for cfmm in all_cfmms]

    cfmm_tx_cost = sum([eta[i] * all_tx_cost[i] for i in range(len(all_cfmms))])

    deltas = [cp.Variable(len(cfmm), nonneg=True) for cfmm in all_cfmms]
    lambdas = [cp.Variable(len(cfmm), nonneg=True) for cfmm in all_cfmms]

    psi = cp.sum(
        [
            A_i @ (LAMBDA - DELTA)
            for A_i, DELTA, LAMBDA in zip(mapping_matrices, deltas, lambdas)
        ]
    )

    # Creating the objective
    objective = cp.Maximize(psi[all_tokens.index(obj_token)] - cfmm_tx_cost)

    # Creating the reserves
    new_reserves_list = [
        R + gamma * D - L
        for R, gamma, D, L in zip(all_reserves, all_fees, deltas, lambdas)
    ]

    # Creating the constraints
    constraints = [psi + current_assets >= 0]

    # Pool constraints
    for i, cfmm in enumerate(all_cfmms):
        new_reserves = new_reserves_list[i]
        old_reserves = all_reserves[i]
        if cfmm.is_transfer:
            constraints.append(cp.sum(new_reserves) >= cp.sum(old_reserves))
            constraints.append(new_reserves >= 0)

        else:
            # In this case we don't have a transfer pool but a regular swap
            constraints.append(cp.geo_mean(new_reserves) >= cp.geo_mean(old_reserves))

    # Forcing delta and lambda to be 0 if eta is 0
    for i, cfmm in enumerate(all_cfmms):
        if eta[i] == 0:
            constraints.append(deltas[i] == 0)
            constraints.append(lambdas[i] == 0)
        else:
            constraints.append(deltas[i] <= MAX_RESERVE)

    # Set up and solve the problem
    problem = cp.Problem(objective, constraints)
    problem.solve(verbose=False, solver="CLARABEL", qcp=False)

    return deltas, lambdas, psi, objective

if __name__ == "__main__": 
    chains: dict[str, list[str]] = {
    "ETHEREUM": ["WETH", "USDC", "SHIBA"],
    'CENTAURI': [],
    "OSMOSIS": ["ATOM","SCRT"],
    }

    sim_env = OrderRoutingSimulationEnvironment(
        center_node='CENTAURI', 
        max_reserve=1e10, 
        chains=chains
    )
    
    # Solving with unknnown eta values
    origin_token = "WETH"
    number_of_init_tokens = 2000 
    obj_token = "ATOM"


    deltas, lambdas, psi, eta = solve_with_unknown_eta(
        origin_token, 
        number_of_init_tokens, 
        obj_token, 
        sim_env.all_tokens, 
        sim_env.all_cfmms, 
        sim_env.mapping_matrices
    )
    
    # We go through various values of eta and solve the problem for each of them
    t_values = sorted([eta[i].value for i in range(len(sim_env.all_cfmms))])    
    results = {} 
    
    for j in tqdm(range(len(t_values))):
        example_eta = [int(eta[i].value >= t_values[j]) for i in range(len(sim_env.all_cfmms))]
    
        try: 
            deltas, lambdas, psi, objective = solve_with_known_eta(
                origin_token, 
                number_of_init_tokens, 
                obj_token, 
                sim_env.all_tokens, 
                sim_env.all_cfmms, 
                sim_env.mapping_matrices,
                example_eta
            )
            
            results[j] = {
                'deltas': [delta.value for delta in deltas],
                'lambdas': [lambda_.value for lambda_ in lambdas],
                'psi': psi.value,
                'objective': objective.value, 
                'eta': example_eta,
            }
            
        except:
            print(f"Failed for t={t_values[j]}")
            continue
        
    # Getting the best result by looking at the objective value
    best_result_key = max(results, key=lambda x: results[x]['objective'])
    best_result = results[best_result_key]
    
    # Taking the best result and getting the trades
    best_eta = best_result['eta']
    best_deltas = best_result['deltas']
    best_lambdas = best_result['lambdas']
    best_psi = best_result['psi']

    chosen_cfmms = [cfmm for i, cfmm in enumerate(sim_env.all_cfmms) if best_eta[i] == 1]
    chosen_mapping_matrices = [mapping_matrix for i, mapping_matrix in enumerate(sim_env.mapping_matrices) if best_eta[i] == 1]
    chosen_deltas = [delta for i, delta in enumerate(best_deltas) if best_eta[i] == 1]
    chosen_lambdas = [lambda_ for i, lambda_ in enumerate(best_lambdas) if best_eta[i] == 1]
    
    # Creating the swaps from each of these
    swaps = [] 

    for i in range(len(chosen_cfmms)): 
        c = chosen_cfmms[i]
        m = chosen_mapping_matrices[i]
        d = chosen_deltas[i]
        l = chosen_lambdas[i]
        
        trade = m @ (l - d)
        
        tokens_from_trade = get_tokens_from_trade(trade, sim_env.all_tokens)
        
        if tokens_from_trade[c.tokens[0]] > 0:
            # This means that I got this token out from the CFMM
            out_token = c.tokens[0]
            in_token = c.tokens[1]
        else: 
            out_token = c.tokens[1]
            in_token = c.tokens[0]
        
        if c.is_transfer: 
            swap_type = 'transfer'
        else: 
            swap_type = 'swap'
            
        swaps.append(Swap(in_token, abs(tokens_from_trade[in_token]), out_token, abs(tokens_from_trade[out_token]), swap_type))
        
    for swap in swaps: 
        print(swap)