# CVXPY Angeris helper algorithm which more or less independnt of exact implementation

import math

import numpy as np
from anytree import RenderTree
from loguru import logger

from simulation.routers.data import (
    AllData,
    AssetPairsXyk,
    AssetTransfers,
    Ctx,
    Exchange,
    Input,
    SingleInputAssetCvmRoute,
    Spawn,
)

from .data import CvxpySolution, VenuesSnapshot


def cvxpy_to_data(
    input: Input,
    data: AllData,
    ctx: Ctx,
    raw_solution: CvxpySolution,
    ratios=None,
) -> SingleInputAssetCvmRoute:
    """_summary_
    Converts Angeris CVXPY result to executable route.
    Receives solution along with all data and context.
    Clean up near zero trades.
    Make `delta-lambda` to be just single trades over venues.
    Start building fork-join supported route tree tracking venue.
      Find starter node and recurse with minus from input matrix (loops covered).
    Visualize.
    """

    index_of_input = data.index_of_token(input.in_token_id)
    index_of_output = data.index_of_token(input.out_token_id)

    if not ratios or len(ratios) == 0:
        ratios = {asset_id: 1 for asset_id in data.all_tokens}

    raw_solution.psi[index_of_output].value / ratios[input.out_token_id]
    solution_input = -raw_solution.psi[index_of_input].value / ratios[input.in_token_id]
    if ratios is None:
        ratios = {asset_id: 1 for asset_id in data.all_tokens}

    # if solution_input < in_amount * ctx.input_consumed_ratio:
    #     raise Exception(f"input {in_amount} > solution_input {solution_input}")
    # if solution_output < out_amount:
    #     raise Exception(f"output {out_amount} > solution_output {solution_output}")

    _etas, trades_raw = parse_total_traded(ctx, raw_solution)
    total_trades = into_venue_snapshots(data, ratios, trades_raw)
    # assert any([trade for trade in total_trades])
    # # balances
    # in_tokens = defaultdict(int)
    # out_tokens = defaultdict(int)
    # for trade in total_trades:
    #     if trade:
    #         in_tokens[trade.in_token] += trade.in_amount
    #         out_tokens[trade.out_token] += trade.out_amount

    for trade in total_trades:
        if trade:
            logger.info(f"trade {trade}")

    depth = 0

    def snapshots_to_route(current: VenuesSnapshot, depth, input: Input, ctx: Ctx):
        """_summary_
        Add nodes until burn all input from snapshots.

        If depth is more than some level or amount is close to received, and asset in current is received,
        stop iterating.
        """
        depth += 1
        from_big_to_small = sorted(
            [
                trade
                for trade in total_trades
                if trade and trade.in_asset_id == current.out_asset_id and trade.in_amount > 0 and trade.out_amount > 0
            ],
            key=lambda x: x.in_amount,
            reverse=True,
        )

        if sum([trade.out_amount for trade in total_trades if trade and trade.out_asset_id == input.out_token_id]) <= 0:
            logger.error("WTF")
            raise Exception(total_trades)
            return
        if depth > ctx.depth_of_route:
            logger.trace("depth of route limit reached")
            if input.out_token_id == current.out_asset_id:
                logger.trace("ended because of depth")
                return

        logger.error("========================================")
        if any(from_big_to_small):
            if current.out_amount <= 0:
                raise Exception("must not get to here but stop one iteration earlier")
            for snapshot in from_big_to_small:
                logger.error(f"======================================== {snapshot}")
                logger.debug(f"snapshot will={snapshot};depth={depth}")
                if snapshot.out_amount <= 0 or snapshot.in_amount <= 0:
                    continue
                traded_in_amount = min(current.out_amount, snapshot.in_amount)
                if traded_in_amount <= 0:
                    logger.warning(f"cannot trade nothing for in={current} via={snapshot} ")
                    continue

                received_out_amount = min(
                    snapshot.out_amount, traded_in_amount * snapshot.out_amount / snapshot.in_amount
                )
                if received_out_amount <= 0:
                    continue

                logger.debug(f"expected_out_amount {received_out_amount}; traded in amount = {traded_in_amount} ")
                snapshot.out_amount -= received_out_amount
                snapshot.in_amount -= traded_in_amount
                current.out_amount -= traded_in_amount

                next_trade = VenuesSnapshot(
                    name="venue",
                    in_amount=traded_in_amount,
                    out_amount=received_out_amount,
                    venue_index=snapshot.venue_index,
                    in_asset_id=snapshot.in_asset_id,
                    out_asset_id=snapshot.out_asset_id,
                    parent=current,
                )
                logger.info(f"next_trade {next_trade}")
                snapshots_to_route(next_trade, depth, input, ctx)

    start = VenuesSnapshot(
        name="input",
        in_amount=-1,
        venue_index=-1,
        in_asset_id=-1,
        out_amount=int(solution_input),
        out_asset_id=input.in_token_id,
    )
    snapshots_to_route(start, depth, input, ctx)

    for pre, _fill, node in RenderTree(start):
        logger.debug(f"{pre} {node}")

    def simulate_route(current_snapshot: VenuesSnapshot):
        """
        Go over route from snapshots and simulate to set numbers to be exact
        """
        if current_snapshot.name == "input":
            subs = []
            if current_snapshot.children:
                for child in current_snapshot.children:
                    sub = simulate_route(child)
                    subs.append(sub)

            return SingleInputAssetCvmRoute(
                in_amount=current_snapshot.out_amount,
                next=subs,
            )
        elif current_snapshot.name == "venue":
            subs = []
            if current_snapshot.children:
                for child in current_snapshot.children:
                    sub = simulate_route(child)
                    subs.append(sub)
            venue = data.venue_by_index(current_snapshot.venue_index)
            in_asset_id = str(current_snapshot.in_asset_id)
            out_asset_id = str(current_snapshot.out_asset_id)
            in_asset_amount = int(math.floor(current_snapshot.frozen.in_amount))
            out_asset_amount = int(math.floor(current_snapshot.frozen.out_amount))
            if isinstance(venue, AssetPairsXyk):
                return Exchange(
                    in_asset_id=in_asset_id,
                    out_asset_id=out_asset_id,
                    in_asset_amount=in_asset_amount,
                    out_asset_amount=out_asset_amount,
                    pool_id=str(venue.pool_id),
                    next=subs,
                )
            elif isinstance(venue, AssetTransfers):
                return Spawn(
                    in_asset_id=in_asset_id,
                    out_asset_id=out_asset_id,
                    in_asset_amount=in_asset_amount,
                    out_asset_amount=out_asset_amount,
                    next=subs,
                )
        else:
            raise Exception("Unknown venue type")

    return simulate_route(start)


def into_venue_snapshots(data, ratios, trades_raw) -> list[VenuesSnapshot]:
    """_summary_
    Converts CVXPY Angeris solutions to something more usable for final route conversion with original scaling
    """
    total_trades = []

    for i, raw_trade in enumerate(trades_raw):
        if np.abs(raw_trade[0]) > 0:
            [token_index_a, token_index_b] = data.venues_tokens[i]
            if raw_trade[0] < 0:
                total_trades.append(
                    VenuesSnapshot(
                        name="any",
                        in_asset_id=token_index_a,
                        in_amount=-raw_trade[0] / ratios[token_index_a],
                        out_asset_id=token_index_b,
                        out_amount=raw_trade[1] / ratios[token_index_b],
                        venue_index=i,
                    )
                )
            else:
                total_trades.append(
                    VenuesSnapshot(
                        name="any",
                        in_asset_id=token_index_b,
                        in_amount=-raw_trade[1] / ratios[token_index_b],
                        out_asset_id=token_index_a,
                        out_amount=raw_trade[0] / ratios[token_index_a],
                        venue_index=i,
                    )
                )
        else:
            total_trades.append(None)

    return total_trades


def parse_total_traded(ctx: Ctx, result: CvxpySolution) -> tuple[any, list]:
    etas = result.eta_values
    deltas = result.delta_values
    lambdas = result.lambda_values

    # clean up near zero trades
    trades_raw = []
    for i in range(result.count):
        lambdas[i] - deltas[i]
        if etas[i] < ctx.minimal_trading_probability:
            logger.error(f"by ETA venue={i} delta={deltas[i]} lambda={lambdas[i]} eta={etas[i]}")
            etas[i] = 0
            deltas[i] = np.zeros(len(deltas[i]))
            lambdas[i] = np.zeros(len(lambdas[i]))
        # elif np.abs(raw_trade[0]) < ctx.minimal_tradeable_number and np.abs(raw_trade[1]) < ctx.minimal_tradeable_number:
        #     logger.error(f"by DELTA LAMBDA venue={i} delta={deltas[i]} lambda={lambdas[i]} eta={etas[i]}")
        #     etas[i] = 0
        #     deltas[i] = np.zeros(len(deltas[i]))
        #     lambdas[i] = np.zeros(len(lambdas[i]))
        else:
            logger.error(f"by RETAINED venue={i} delta={deltas[i]} lambda={lambdas[i]} eta={etas[i]}")
        trades_raw.append(lambdas[i] - deltas[i])
    # for i in range(result.count):
    #     if etas[i] >= 1.0 - ctx.minimal_amount:
    #         etas[i] = None
    return etas, trades_raw
