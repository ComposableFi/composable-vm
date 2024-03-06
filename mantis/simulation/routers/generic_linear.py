# solves using NLP optimization (or what best underlying engine decides)
# Models cross chain transfers as fees as """pools"""
# Uses decision variables to decide if to do Transfer to tap pool or not.
# Is generic as possible, can be slow

from dataclasses import dataclass
from typing import Union

import cvxpy as cp
import numpy as np
from loguru import logger
from mpire import WorkerPool

from simulation.routers.angeris_cvxpy import CvxpySolution
from simulation.routers.data import AllData, Ctx, Input


@dataclass
class HeuristicParams:
    no_constant_fees: bool = False
    relaxed: bool = False


# prepares data for solving and outputs raw solution from underlying engine
def solve(
    all_data: AllData,
    input: Input,
    ctx: Ctx,
    force_eta: list[Union[int, None]] = None,
    forced_max: list[Union[int, float]] = None,
    relaxed=False,
    continuous=False,
) -> CvxpySolution:
    if not input.max:
        raise NotImplementedError("'max' value on input is not supported to be False yet")

    if force_eta:
        logger.info(f"using forcing etas {force_eta} for solution")

    mi = (
        force_eta is not None
        and len([eta for eta in force_eta if not eta == 0]) <= ctx.mi_for_venue_count
        and ctx.integer
    )
    if mi:
        logger.info("asking mixed integer solution")
    else:
        logger.debug("Using optimization: continuous optimization")

    # using this need second round scale in to fit integer solution
    # milp = lambda x: int(x) if mi else x

    # initial input assets
    index_of_input_token = all_data.index_of_token(input.in_token_id)
    all_data.index_of_token(input.out_token_id)
    current_assets = np.full((all_data.tokens_count), int(0))
    current_assets[index_of_input_token] = input.in_amount
    # received_assets_index = np.full((all_data.tokens_count), int(0))
    # tendered_assets_index = np.full((all_data.tokens_count), int(0))
    # received_assets_index[index_of_output_asset] = 1
    # tendered_assets_index[index_of_input_token] = 1

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

    # tendered (given) amount of reserves
    deltas = [cp.Variable(A_i.shape[1], integer=mi) for A_i in A]

    # received (wanted) amountsы of reserves
    lambdas = [cp.Variable(A_i.shape[1], integer=mi) for A_i in A]
    # indicates tx or not for given pool
    # zero means no TX it sure
    etas = cp.Variable(
        all_data.venues_count,
        boolean=(ctx.integer or mi) and not continuous,
    )

    # network trade vector - net amount received over all venues(transfers/exchanges)
    psi = cp.sum([A_i @ (LAMBDA - DELTA) for A_i, DELTA, LAMBDA in zip(A, deltas, lambdas)])

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
    phi = [
        R + gamma_i * D - L  # * `fee out` to add
        for R, gamma_i, D, L in zip(reserves, all_data.venues_proportional_reductions, deltas, lambdas)
    ]

    # Trading function constraints
    constraints = [
        psi + current_assets >= 0,
    ]
    # if not strict:
    #     constraints.append(psi[index_of_input_token] <= -0.95 * input.in_amount)

    if not continuous:
        input_forced_etas = np.full((all_data.venues_count), int(0))
        output_forced_etas = np.full((all_data.venues_count), int(0))
        for i, venue in enumerate(all_data.venues):
            if venue.in_asset_id == input.in_token_id or venue.out_asset_id == input.in_token_id:
                input_forced_etas[i] = 1
            if venue.in_asset_id == input.out_token_id or venue.out_asset_id == input.out_token_id:
                output_forced_etas[i] = 1
        constraints.append(cp.sum(etas) >= 1)
        constraints.append(cp.sum(cp.multiply(etas, input_forced_etas)) >= 1)
        constraints.append(cp.sum(cp.multiply(etas, output_forced_etas)) >= 1)

    if not relaxed:
        constraints.append(psi[index_of_input_token] >= -input.in_amount)
        # constraints.append(psi[index_of_output_asset] >= input.out_amount)

    # if not continuous and not etas_counter:
    constraints.append(cp.sum(etas) <= ctx.maximal_venues_count)

    # so we do not want to retain any other tokens on road
    # so there can be trade over pools which give some side token not to trade, which is useless
    # should we burn it or not?
    # if not relaxed:
    #     for token in all_data.all_tokens:
    #         if token != input.in_token_id and token != input.out_token_id:
    #             constraints.append(psi[all_data.index_of_token(token)] >= 0)

    # input to venue can be only positive
    for delta_i in deltas:
        constraints.append(delta_i >= 0)

    # output of venue can be only positive
    for lambda_i in lambdas:
        constraints.append(lambda_i >= 0)

    for eta_i in etas:
        constraints.append(eta_i >= 0)
        if continuous:
            constraints.append(eta_i <= 1)

    # Pool constraint (Uniswap v2 like)
    for x in all_data.asset_pairs_xyk:
        i = all_data.get_index_in_all(x)
        if reserves[i][0] <= ctx.minimal_venued_amount or reserves[i][1] <= ctx.minimal_venued_amount:
            constraints.append(deltas[i] == 0)
            constraints.append(lambdas[i] == 0)
            reserves[i][0] = 0
            reserves[i][1] = 0
        else:
            constraints.append(cp.geo_mean(phi[i]) >= cp.geo_mean(reserves[i]))

    # Pool constraint for cross chain transfer transfer (constant sum)
    for x in all_data.asset_transfers:
        i = all_data.get_index_in_all(x)
        # realistically that is depends on side source vs target
        # source chain can mint any amount up to total issuance
        # while target chain can back only limited amount escrowed
        # so on source chain limit is current total issuance not locked on that chain
        if reserves[i][0] <= ctx.minimal_venued_amount or reserves[i][1] <= ctx.minimal_venued_amount:
            constraints.append(deltas[i] == 0)
            constraints.append(lambdas[i] == 0)
            reserves[i][0] = 0
            reserves[i][1] = 0
        else:
            constraints.append(cp.sum(phi[i]) >= cp.sum(reserves[i]))
            constraints.append(phi[i] >= 0)

    # Enforce deltas depending on pass or not pass variable

    for i in range(all_data.venues_count):
        if force_eta is not None and force_eta[i] is not None:
            assert etas[i] is not None
            constraints.append(etas[i] == force_eta[i])
            if force_eta[i] == 0:
                constraints.append(deltas[i] == 0)
                constraints.append(lambdas[i] == 0)
        elif reserves[i][0] <= ctx.minimal_venued_amount or reserves[i][1] <= ctx.minimal_venued_amount:
            constraints.append(etas[i] == 0)
            constraints.append(deltas[i] == 0)
            constraints.append(lambdas[i] == 0)
        else:
            max_a_asset_reserve = all_data.maximal_reserves_of(all_data.venues_tokens[i][0])
            max_b_asset_reserve = all_data.maximal_reserves_of(all_data.venues_tokens[i][1])
            if max_a_asset_reserve <= ctx.minimal_venued_amount or max_b_asset_reserve <= ctx.minimal_venued_amount:
                logger.info("warning:: mantis::simulation::router:: trading with zero liquid amount of token")

            max_delta = (
                forced_max[i]
                if forced_max is not None and len(forced_max) > 0
                else [max_a_asset_reserve, max_b_asset_reserve]
            )
            logger.debug(f"using {max_delta} to cap delta for {i}")
            constraints.append(deltas[i] <= etas[i] * max_delta)

    problem = cp.Problem(obj, constraints)
    problem.solve(
        verbose=ctx.debug,
        solver=cp.SCIP,
        qcp=False,
        gp=False,
    )

    solution = CvxpySolution(
        deltas=deltas,
        lambdas=lambdas,
        psi=psi,
        etas=etas,
        problem=problem,
        eta_values=np.array([x.value for x in etas]),
        input=input,
        data=all_data,
    )
    if not continuous:
        solution.cut_unconditional()
        solution.cut_small_numbers()
        solution.cut_using_oracles()
        solution.ensure_bug_trades_pay_fee()
    solution.verify(ctx)
    return solution


def route(
    input: Input,
    all_data: AllData,
    ctx: Ctx = Ctx(),
):
    """
    solves and decide if routable
    """
    logger.info(f"routing requested for {input}")
    # ctx.sequential = True
    initial_solutions = []
    continuous: CvxpySolution = None
    with WorkerPool(n_jobs=1 if ctx.sequential else 3) as pool:
        strict = (all_data, input, ctx, None, None, False)
        relaxed = (all_data, input, ctx, None, None, True)
        approximate = (all_data, input, ctx, None, None, True, True)
        strict = pool.apply_async(solve, strict, task_timeout=5)
        relaxed = pool.apply_async(solve, relaxed, task_timeout=5)
        approximate = pool.apply_async(solve, approximate, task_timeout=5)

        strict.wait()
        relaxed.wait()
        approximate.wait()

        try:
            strict = strict.get()
            initial_solutions.append(strict)
        except Exception as e:
            logger.error(f"Failed to solve strict {e}")
        try:
            relaxed = relaxed.get()
            initial_solutions.append(relaxed)
        except Exception as e:
            logger.error(f"Failed to solve relaxed {e}")

        try:
            continuous = approximate.get()
        except Exception as e:
            logger.error(f"Failed to solve approximate {e}")

    if len(initial_solutions) == 0:
        raise Exception("all solvers failed, solution considered infeasible")

    forced = []
    for solution in initial_solutions:
        forced.append((solution.forced_etas, solution.to_forced_max(all_data, ctx)))

    lucks_forced = []
    if continuous is not None:
        lucks = [i for i, eta in sorted(enumerate(continuous.eta_values), reverse=True, key=lambda x: x[1])]

        for i in range(1, min(10, len(lucks))):
            luck_etas = [0] * len(lucks)
            for j in range(i):
                luck_etas[lucks[j]] = None

            lucks_forced.append((luck_etas, None))

    parameters = []
    forced_solutions = []
    for force_eta, forced_max in forced:
        parameters.append((all_data, input, ctx, force_eta, forced_max, False, False))

    for force_eta, forced_max in lucks_forced:
        parameters.append((all_data, input, ctx, force_eta, continuous.to_forced_max(all_data, ctx), False, False))

    with WorkerPool(n_jobs=1 if ctx.sequential else 10) as pool:
        results = []
        for parameter in parameters:
            results.append(pool.apply_async(solve, parameter, task_timeout=5))
        for result in results:
            result.wait()
        for result in results:
            try:
                solution = result.get()
                forced_solutions.append(solution)
            except Exception as e:
                logger.error(f"Failed to solve with forced solution {e}")

    initial_solutions.extend(forced_solutions)

    if len(initial_solutions) == 0:
        raise Exception("not of solutions feet")
    return initial_solutions
