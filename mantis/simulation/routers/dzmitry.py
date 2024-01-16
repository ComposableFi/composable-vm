# solves using convex optimization
import numpy as np
import cvxpy as cp

MAX_RESERVE = 1e10

from simulation.routers.data import AllData, Input, TAssetId, TNetworkId

# prepares data for solving and outputs raw solution from underlying engine
def solve(
    all_data: AllData,
    all_cfmms: list[tuple[TAssetId, TAssetId]],
    reserves: list[np.ndarray[np.float64]],
    cfmm_tx_cost: list[float],
    fees: list[float],
    ibc_pools: int,
    input : Input,
    force_eta: list[float] = None,
):
    # Build local-global matrices
    count_tokens = len(all_data.tokens_count)
    count_cfmms = len(all_cfmms)

    current_assets = np.zeros(count_tokens)  # Initial assets
    current_assets[all_data.index_of_token(input.in_token_id)] = input.in_amount

    A = []
    for cfmm in all_cfmms:
        n_i = len(cfmm)  # Number of tokens in pool (default to 2)
        A_i = np.zeros((count_tokens, n_i))
        for i, token in enumerate(cfmm):
            A_i[all_data.index_of_token(token), i] = 1
        A.append(A_i)

    # Build variables
    
    # tendered (given) amount
    deltas = [cp.Variable(len(l), nonneg=True) for l in all_cfmms]
    
    # received (wanted) amounts
    lambdas = [cp.Variable(len(l), nonneg=True) for l in all_cfmms]
    
    eta = cp.Variable(
        count_cfmms, 
        nonneg=True, 
        # boolean=True, # Problem is mixed-integer, but candidate QP/Conic solvers ([]) are not MIP-capable.
    )  # Binary value, indicates tx or not for given pool

    # network trade vector - net amount received over all trades(transfers/exchanges)
    psi = cp.sum([A_i @ (LAMBDA - DELTA) for A_i, DELTA, LAMBDA in zip(A, deltas, lambdas)])
    
    # Objective is to trade number_of_init_tokens of asset origin_token for a maximum amount of asset objective_token
    obj = cp.Maximize(psi[all_data.index_of_token(input.out_token_id)] - eta @ cfmm_tx_cost)

    # Reserves after trade
    new_reserves = [
        R + gamma_i * D - L for R, gamma_i, D, L in zip(reserves, fees, deltas, lambdas)
    ]

    # Trading function constraints
    constrains = [
        psi + current_assets >= 0,
    ]

    # Pool constraint (Uniswap v2 like)
    for i in range(count_cfmms - ibc_pools):
        constrains.append(cp.geo_mean(new_reserves[i]) >= cp.geo_mean(reserves[i]))

    # Pool constraint for IBC transfer (constant sum)
    # NOTE: Ibc pools are at the bottom of the cfmm list
    for i in range(count_cfmms - ibc_pools, count_cfmms):
        constrains.append(cp.sum(new_reserves[i]) >= cp.sum(reserves[i]))
        constrains.append(new_reserves[i] >= 0)

    # Enforce deltas depending on pass or not pass variable
    # MAX_RESERVE should be big enough so delta <<< MAX_RESERVE
    for i in range(count_cfmms):
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

    print(
        f"\033[1;91mTotal amount out: {psi.value[all_data.index_of_token(input.out_token_id)]}\033[0m"
    )

    for i in range(count_cfmms):
        print(
            f"Market {all_cfmms[i][0]}<->{all_cfmms[i][1]}, delta: {deltas[i].value}, lambda: {lambdas[i].value}, eta: {eta[i].value}",
        )
    
    # deltas[i] - how much one gives to pool i
    # lambdas[i] - how much one wants to get from pool i
    return deltas, lambdas, psi, eta


# solves and decide if routable
def route(input: Input, all_data: AllData,):
    
    _deltas, _lambdas, psi, n = solve(
        all_data, 
        all_cfmms, 
        reserves, 
        cfmm_tx_cost, 
        fees, 
        ibc_pools, 
        input,
        )

    to_look_n: list[float] = []
    for i in range(len(all_cfmms)):
        to_look_n.append(n[i].value)

    _max = 0
    for t in sorted(to_look_n):
        try:
            d2, l2, p2, n2 =  solve(
                all_data,
                all_cfmms,
                reserves,
                cfmm_tx_cost,
                fees,
                ibc_pools,
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
                all_cfmms,
                reserves,
                cfmm_tx_cost,
                fees,
                ibc_pools,
                input.in_token_id,
                input.in_amount,
                input.out_token_id,
                eta,
            )

        except:
            continue

    print("---")
    deltas, lambdas, psi, eta = solve(
                    all_data,
                    all_cfmms,
                    reserves,
                    cfmm_tx_cost,
                    fees,
                    ibc_pools,
                    input,
                    eta,
                )
    m = len(all_cfmms)
    for i in range(m):
        print(
            f"Market {all_cfmms[i][0]}<->{all_cfmms[i][1]}, delta: {deltas[i].value}, lambda: {lambdas[i].value}, eta: {eta[i].value}",
        )

    print(psi.value[all_data.index_of_token(input.out_token_id)],lastp_value)
    # basically, we have matrix where rows are in tokens (DELTA)
    # columns are outs (LAMBDA)
    # so recursively going DELTA-LAMBDA and subtracting values from cells
    # will allow to build route with amounts 
    return (psi.value[all_data.index_of_token(input.out_token_id)],lastp_value)