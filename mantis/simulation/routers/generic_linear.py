# solves using NLP optimization (or what best underlying engine decides)
# Models cross chain transfers as fees as """pools"""
# Uses decision variables to decide if to do Transfer to tap pool or not.
# Is generic as possible, can be slow

import copy
import sys
from typing import Union

import cvxpy as cp
import numpy as np
from loguru import logger

from simulation.routers.angeris_cvxpy import CvxpySolution, parse_trades
from simulation.routers.data import AllData, Ctx, Input

logger.remove()
logger.add(sys.stderr, colorize=True, format="<green>{time}</green> <level>{message}</level>")

# prepares data for solving and outputs raw solution from underlying engine
def solve(
    all_data: AllData,
    input: Input,
    ctx: Ctx,
    force_eta: list[Union[int, None]] = None,
) -> CvxpySolution:
    if not input.max:
        raise NotImplementedError(
            "'max' value on input is not supported to be False yet"
        )

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
    mi = (
        force_eta is not None
        and len([eta for eta in force_eta if not eta == 0]) <= ctx.mi_for_venue_count
    )

    # tendered (given) amount of reserves
    deltas = [cp.Variable(A_i.shape[1], integer=mi) for A_i in A]

    # received (wanted) amountsы of reserves
    lambdas = [cp.Variable(A_i.shape[1], integer=mi) for A_i in A]
    # indicates tx or not for given pool
    # zero means no TX it sure
    etas = cp.Variable(
        all_data.venues_count,
        # integer = ctx.integer,
        boolean=ctx.integer,
    )

    # network trade vector - net amount received over all venues(transfers/exchanges)
    psi = cp.sum(
        [A_i @ (LAMBDA - DELTA) for A_i, DELTA, LAMBDA in zip(A, deltas, lambdas)]
    )

    assert len(A) == all_data.venues_count
    assert len(reserves) == all_data.venues_count
    assert len(current_assets) == len(all_data.all_tokens)

    # Objective is to trade number_of_init_tokens of asset origin_token for a maximum amount of asset objective_token
    obj = cp.Maximize(
        psi[all_data.index_of_token(input.out_token_id)]
        # so it will set ZERO to venues it wants to trades
        - etas @ all_data.venue_fixed_costs_in(input.out_token_id)
    )  # divide costs by target price in usd

    # Reserves after trade
    new_reserves = [
        R + gamma_i * D - L  # * `fee out` to add
        for R, gamma_i, D, L in zip(
            reserves, all_data.venues_proportional_reductions, deltas, lambdas
        )
    ]

    # Trading function constraints
    constraints = [
        psi + current_assets >= 0,
    ]

    # input to venue can be only positive
    for delta_i in deltas:
        constraints.append(delta_i >= 0)

    # output of venue can be only positive
    for lambda_i in lambdas:
        constraints.append(lambda_i >= 0)

    for eta_i in etas:
        constraints.append(eta_i >= 0)
        if ctx.integer:
            constraints.append(eta_i <= 1)

    # Pool constraint (Uniswap v2 like)
    for x in all_data.asset_pairs_xyk:
        i = all_data.get_index_in_all(x)
        if reserves[i][0] == 0 or reserves[i][1] == 0:
            print("zeros")
            # constraints.append(deltas[i] == 0)
            # constraints.append(lambdas[i] == 0)
        else:
            constraints.append(cp.geo_mean(new_reserves[i]) >= cp.geo_mean(reserves[i]))

    # Pool constraint for cross chain transfer transfer (constant sum)
    for x in all_data.asset_transfers:
        i = all_data.get_index_in_all(x)
        # realistically that is depends on side source vs target
        # source chain can mint any amount up to total issuance
        # while target chain can back only limited amount escrowed
        # so on source chain limit is current total issuance not locked on that chain
        if reserves[i][0] == 0 or reserves[i][1] == 0:
            print("zeros")
            # constraints.append(deltas[i] == 0)
            # constraints.append(lambdas[i] == 0)
        else:
            constraints.append(cp.sum(new_reserves[i]) >= cp.sum(reserves[i]))
            constraints.append(new_reserves[i] >= 0)

    # Enforce deltas depending on pass or not pass variable

    for i in range(all_data.venues_count):
        if force_eta is not None and force_eta[i] is not None:
            constraints.append(etas[i] == force_eta[i])
            if force_eta[i] == 0:
                constraints.append(deltas[i] == 0)
                constraints.append(lambdas[i] == 0)
        else:
            issuance = 1
            token_a_global = issuance * all_data.global_reservers_of(
                all_data.venues_tokens[i][0]
            )
            token_b_global = issuance * all_data.global_reservers_of(
                all_data.venues_tokens[i][1]
            )
            if (
                token_a_global <= ctx.minimal_amount
                or token_b_global <= ctx.minimal_amount
            ):
                logger.info(
                    "warning:: mantis::simulation::router:: trading with zero liquid amount of token"
                )
            constraints.append(deltas[i] <= etas[i] * [token_a_global, token_b_global])
            # constraints.append(cp.multiply(deltas[i], etas[i]) >= deltas[i])
    # Set up and solve problem
    problem = cp.Problem(obj, constraints)
    # success: CLARABEL,
    # failed: ECOS, GLPK, GLPK_MI, CVXOPT, SCIPY, CBC, SCS
    #
    # GLOP, SDPA, GUROBI, OSQP, CPLEX, MOSEK, , COPT, XPRESS, PIQP, PROXQP, NAG, PDLP, SCIP, DAQP
    problem.solve(
        verbose=ctx.debug,
        solver=cp.SCIP,
        qcp=False,
        gp=False,
    )

    if problem.status not in ["optimal", "optimal_inaccurate"]:
        raise Exception(f"Problem status: {problem.status}")

    logger.info(
        f"\033[1;91mTotal amount out: {psi.value[all_data.index_of_token(input.out_token_id)]}\033[0m"
    )

    for i in range(all_data.venues_count):
        logger.info(
            f"Market {all_data.assets_for_venue(i)} {all_data.all_reserves[i][0]}<->{all_data.all_reserves[i][1]}, delta: {deltas[i].value}, lambda: {lambdas[i].value}, eta: {etas[i].value}",
        )

    return CvxpySolution(
        deltas=deltas,
        lambdas=lambdas,
        psi=psi,
        etas=etas,
        problem=problem,
    )


def prepare_data(input: Input, all_data: AllData):
    """_summary_
    Prepares data usable specifically by this solver from general input
    """
    pass


def route(
    input: Input,
    all_data: AllData,
    ctx: Ctx = Ctx(),
):
    """
    solves and decide if routable
    """
    if ctx.debug:
        logger.info("first run")
    initial_solution = solve(
        all_data,
        input,
        ctx,
    )
    forced_etas, original_trades = parse_trades(ctx, initial_solution)
    if ctx.debug:
        logger.info("forced_etas", forced_etas)
        logger.info("original_trades", original_trades)
    forced_eta_solution = solve(all_data, input, ctx, forced_etas)
    solution = copy.deepcopy(forced_eta_solution)
    return solution
