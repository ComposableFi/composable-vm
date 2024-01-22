# solves using NLP optimization (or what best underlying engine decides)
# Models cross chain transfers as fees as """pools"""
# Uses decision variables to decide if to do Transfer to tap pool or not.

import numpy as np
import cvxpy as cp

from simulation.routers.data import AllData, Input, TId, TNetworkId, Ctx


# prepares data for solving and outputs raw solution from underlying engine
def solve(
    all_data: AllData,
    input: Input,
    ctx : Ctx,
    force_eta: list[float] = None,
):
    # initial input assets

    current_assets = np.full((all_data.tokens_count), int(0))
    current_assets[all_data.index_of_token(input.in_token_id)] = input.in_amount

    reserves = all_data.all_reserves

    # build local-global matrices
    A = []

    for x in all_data.asset_pairs_xyk:
        n_i = 2  # number of tokens in transfer
        A_i = np.full((all_data.tokens_count, n_i), int(0))
        A_i[all_data.index_of_token(x.in_asset_id), 0] = 1
        A_i[all_data.index_of_token(x.out_asset_id), 1] = 1
        A.append(A_i)

    for x in all_data.asset_transfers:
        n_i = 2  # number of tokens in pool
        A_i = np.full((all_data.tokens_count, n_i), int(0))
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
        # boolean=True, # Problem is mixed-integer
    )
    assert all_data.venues_count == eta.shape[0]

    # network trade vector - net amount received over all trades(transfers/exchanges)
    psi = cp.sum(
        [A_i @ (LAMBDA - DELTA) for A_i, DELTA, LAMBDA in zip(A, deltas, lambdas)]
    )

    assert len(A) == all_data.venues_count
    assert len(reserves) == all_data.venues_count
    assert len(current_assets) == len(all_data.all_tokens)

    # Objective is to trade number_of_init_tokens of asset origin_token for a maximum amount of asset objective_token
    obj = cp.Maximize(
        psi[all_data.index_of_token(input.out_token_id)]
        - eta @ all_data.venue_fixed_costs_in(input.out_token_id)
    )  # divide costs by target price in usd

    # Reserves after trade
    new_reserves = [
        R + gamma_i * D - L
        for R, gamma_i, D, L in zip(
            reserves, all_data.venues_proportional_reductions, deltas, lambdas
        )
    ]

    # Trading function constraints
    constraints = [
        psi + current_assets >= 0,
    ]

    # Pool constraint (Uniswap v2 like)
    for x in all_data.asset_pairs_xyk:
        i = all_data.get_index_in_all(x)
        constraints.append(cp.geo_mean(new_reserves[i]) >= cp.geo_mean(reserves[i]))

    # Pool constraint for cross chain transfer transfer (constant sum)
    for x in all_data.asset_transfers:
        i = all_data.get_index_in_all(x)
        constraints.append(cp.sum(new_reserves[i]) >= cp.sum(reserves[i]))
        constraints.append(new_reserves[i] >= 0)

    # Enforce deltas depending on pass or not pass variable
    for i in range(all_data.venues_count):
        if force_eta:
            constraints.append(eta[i] == force_eta[i])
        if force_eta and force_eta[i] == 0:
            # Forcing delta and lambda to be 0 if eta is 0
            constraints.append(deltas[i] == 0)
            constraints.append(lambdas[i] == 0)
        else: 
            # MAX_RESERVE should be big enough so delta <<< MAX_RESERVE
            constraints.append(deltas[i] <= eta[i] * ctx.max_reserve)
        
    # Set up and solve problem
    prob = cp.Problem(obj, constraints)
    # success: CLARABEL,
    # failed: ECOS, GLPK, GLPK_MI, CVXOPT, SCIPY, CBC, SCS
    #
    # GLOP, SDPA, GUROBI, OSQP, CPLEX, MOSEK, , COPT, XPRESS, PIQP, PROXQP, NAG, PDLP, SCIP, DAQP
    prob.solve(
        verbose=ctx.debug,
        solver=cp.SCIP,
        qcp=False,
    )

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
def route(
    input: Input,
    all_data: AllData,
    ctx: Ctx = Ctx(),
):
    _deltas, _lambdas, psi, initial_etas = solve(
        all_data,
        input,
        ctx,
        None,
    )
    to_look_n: list[float] = []
    for i in range(all_data.venues_count):
        to_look_n.append(initial_etas[i].value)

    received_max = 0
    for t in sorted(to_look_n):
        try:
            new_delta, new_lambda, new_psi, new_eta = solve(
                all_data,
                input,
                ctx,
                [1 if value <= t else 0 for value in to_look_n],
            )
            if psi.value[all_data.index_of_token(input.out_token_id)] > received_max:
                delta_max, _lambda_max, _psi_max, eta_max = new_delta, new_lambda, new_psi, new_eta
            print("---")
        except:
            continue
    eta = eta_max
    eta_changed = True
    print("---------")
    # received token
    psi_before_zero_delta_elimination = psi.value[all_data.index_of_token(input.out_token_id)]
    while eta_changed:
        try:
            eta_changed = False

            for i, delta in enumerate(delta_max):
                # if input into venue is small, disable using it
                if all(delta_i.value < 1e-04 for delta_i in delta):
                    eta_max[i] = 0
                    eta_changed = True
            delta_max, _lambdas, psi, eta = solve(
                all_data,
                input,
                ctx,
                eta,
            )

        except:
            continue

    print("---")
    deltas, lambdas, psi, eta = solve(
        all_data,
        input,
        ctx,
        eta,
    )
    for i in range(all_data.venues_count):
        print(
            f"Market {all_data.venue(i)}, delta: {deltas[i].value}, lambda: {lambdas[i].value}, eta: {eta[i].value}",
        )

    print(psi.value[all_data.index_of_token(input.out_token_id)], psi_before_zero_delta_elimination)
    # basically, we have matrix where rows are in tokens (DELTA)
    # columns are outs (LAMBDA)
    # so recursively going DELTA-LAMBDA and subtracting values from cells
    # will allow to build route with amounts
    return (psi.value[all_data.index_of_token(input.out_token_id)], psi_before_zero_delta_elimination)
