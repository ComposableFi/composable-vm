# solves using NLP optimization (or what best underlying engine decides)
# Models cross chain transfers as fees as """pools"""
# Uses decision variables to decide if to do Transfer to tap pool or not. 

import numpy as np
import cvxpy as cp

MAX_RESERVE = 1e10

from simulation.routers.data import AllData, Input, TId, TNetworkId

# prepares data for solving and outputs raw solution from underlying engine
def solve(
    all_data: AllData,
    input : Input,
    force_eta: list[float] = None,
):    
    # initial input assets
    current_assets = np.zeros(all_data.tokens_count)  
    current_assets[all_data.index_of_token(input.in_token_id)] = input.in_amount

    reserves = all_data.all_reserves
    
    
    # build local-global matrices
    A = []
    
    for x in all_data.asset_pairs_xyk:
        n_i = 2  # number of tokens in transfer
        A_i = np.zeros((all_data.tokens_count, n_i))
        A_i[all_data.index_of_token(x.in_asset_id), 0] = 1
        A_i[all_data.index_of_token(x.out_asset_id), 1] = 1
        A.append(A_i)        
        
    for x in all_data.asset_transfers:
        n_i = 2  # number of tokens in pool
        A_i = np.zeros((all_data.tokens_count, n_i))
        A_i[all_data.index_of_token(x.in_asset_id), 0] = 1
        A_i[all_data.index_of_token(x.out_asset_id), 1] = 1
        A.append(A_i)

    # Build variables
    
    # tendered (given) amount
    deltas = [cp.Variable(A_i.shape[1], nonneg=True) for A_i in A]
    
    # received (wanted) amounts
    lambdas = [cp.Variable(A_i.shape[1], nonneg=True) for A_i in A]
    
    # indicates tx or not for given pool
    # zero means no TX it sure
    eta = cp.Variable(
        all_data.venues_count, 
        nonneg=True, 
        # boolean=True, # Problem is mixed-integer, but candidate QP/Conic solvers ([]) are not MIP-capable.
    )

    # network trade vector - net amount received over all trades(transfers/exchanges)
    psi = cp.sum([A_i @ (LAMBDA - DELTA) for A_i, DELTA, LAMBDA in zip(A, deltas, lambdas)])

    assert(len(A) == all_data.venues_count)
    assert(len(reserves)  == all_data.venues_count)
    assert(all_data.venues_count == eta.shape[0])
    assert(len(current_assets) == len(all_data.all_tokens))
    
    # Objective is to trade number_of_init_tokens of asset origin_token for a maximum amount of asset objective_token
    obj = cp.Maximize(psi[all_data.index_of_token(input.out_token_id)] - eta @ all_data.venue_fixed_costs_in_usd)

    # Reserves after trade
    new_reserves = [
        R + gamma_i * D - L for R, gamma_i, D, L in zip(reserves, all_data.venues_proportional_reductions, deltas, lambdas)
    ]

    # Trading function constraints
    constrains = [
        psi + current_assets >= 0,
    ]

    # Pool constraint (Uniswap v2 like)
    
    for x in all_data.asset_pairs_xyk:
        i = all_data.get_index_in_all(x)
        constrains.append(cp.geo_mean(new_reserves[i]) >= cp.geo_mean(reserves[i]))

    # Pool constraint for cross chain transfer transfer (constant sum)
    for x in all_data.asset_transfers:
        i = all_data.get_index_in_all(x)
        constrains.append(cp.sum(new_reserves[i]) >= cp.sum(reserves[i]))
        constrains.append(new_reserves[i] >= 0)

    # Enforce deltas depending on pass or not pass variable
    # MAX_RESERVE should be big enough so delta <<< MAX_RESERVE
    for i in range(all_data.venues_count):
        constrains.append(deltas[i] <= eta[i] * MAX_RESERVE)
        if force_eta:
            constrains.append(eta[i] == force_eta[i])

    # Set up and solve problem
    prob = cp.Problem(obj, constrains)
    # success: CLARABEL,
    # failed: ECOS, GLPK, GLPK_MI, CVXOPT, SCIPY, CBC, SCS
    # 
    # GLOP, SDPA, GUROBI, OSQP, CPLEX, MOSEK, , COPT, XPRESS, PIQP, PROXQP, NAG, PDLP, SCIP, DAQP
    prob.solve(verbose= True, solver = "CLARABEL", qcp = False, )

    print("==========================================================================================")
    print(all_data.index_of_token(input.out_token_id))
    print("==========================================================================================")
    assert(psi != None)
    assert(psi.value != None)
    assert(all_data != None)
    assert(all_data.index_of_token(input.out_token_id) != None)
    
    print(
        f"\033[1;91mTotal amount out: {psi.value[all_data.index_of_token(input.out_token_id)]}\033[0m"
    )

    for i in range(all_data.venues_count):
        print(
            f"Market {all_data.all_reserves[i][0]}<->{all_data.all_reserves[i][1]}, delta: {deltas[i].value}, lambda: {lambdas[i].value}, eta: {eta[i].value}",
        )
    
    # deltas[i] - how much one gives to pool i
    # lambdas[i] - how much one wants to get from pool i
    return deltas, lambdas, psi, eta

def prepare_data(input: Input, all_data: AllData):
    """_summary_
        Prepares data usable specifically by this solver from general input
    """
    pass


# solves and decide if routable
def route(input: Input, all_data: AllData,):
    
    _deltas, _lambdas, psi, n = solve(
        all_data, 
        input,
        )
    raise Exception("buy")
    to_look_n: list[float] = []
    for i in range(all_data.venues_count):
        to_look_n.append(n[i].value)

    _max = 0
    for t in sorted(to_look_n):
        try:
            d2, l2, p2, n2 =  solve(
                all_data,
                input,
                [1 if value <= t else 0 for value in to_look_n],
            )
            if psi.value[all_data.index_of_token(input.out_token_id)] > _max:
                d_max, _l_max, _p_max, n_max = d2, l2, p2, n2 
            print("---")
        except:
            continue
    eta = n_max
    eta_change = True
    print("---------")
    lastp_value = psi.value[all_data.index_of_token(input.out_token_id)]
    while eta_change:
        try:
            eta_change = False

            for idx, delta in enumerate(d_max):
                if all(delta_i.value < 1e-04 for delta_i in delta):
                    n_max[idx] = 0
                    eta_change = True
            d_max, _lambdas, psi, eta = solve(
                all_data,
                input,
                eta,
            )

        except:
            continue

    print("---")
    deltas, lambdas, psi, eta = solve(
                    all_data,
                    input,
                    eta,
                )
    for i in range(all_data.venues_count):
        print(
            f"Market {all_data.all_reserves[i][0]}<->{all_data.all_reserves[i][1]}, delta: {deltas[i].value}, lambda: {lambdas[i].value}, eta: {eta[i].value}",
        )

    print(psi.value[all_data.index_of_token(input.out_token_id)],lastp_value)
    # basically, we have matrix where rows are in tokens (DELTA)
    # columns are outs (LAMBDA)
    # so recursively going DELTA-LAMBDA and subtracting values from cells
    # will allow to build route with amounts 
    return (psi.value[all_data.index_of_token(input.out_token_id)],lastp_value)