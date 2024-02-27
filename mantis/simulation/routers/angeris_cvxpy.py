import math
from typing import Union

import cvxpy as cp
import numpy as np
from anytree import Node, RenderTree
from attr import dataclass
from loguru import logger

from simulation.routers.data import (
    AllData,
    AssetPairsXyk,
    AssetTransfers,
    Ctx,
    Exchange,
    Input,
    Spawn,
)


@dataclass
class CvxpySolution:
    deltas: list[cp.Variable]
    """
    how much one gives to pool i
    """

    lambdas: list[cp.Variable]
    """
    how much one wants to get from pool i
    """

    psi: cp.Variable
    etas: cp.Variable
    problem: cp.Problem

    @property
    def eta_values(self) -> np.ndarray[float]:
        return np.array([x.value for x in self.etas])

    @property
    def delta_values(self) -> list[np.ndarray[float]]:
        return [x.value for x in self.deltas]

    @property
    def lambda_values(self) -> list[np.ndarray[float]]:
        return [x.value for x in self.lambdas]

    def __post_init__(self):
        assert len(self.deltas) > 0
        assert len(self.deltas) == len(self.lambdas) == len(self.eta_values)

    @property
    def count(self):
        return len(self.deltas)

    def received(self, global_index) -> float:
        return self.psi.value[global_index]


@dataclass
class VenueOperation:
    venue_index: int
    in_token: any
    in_amount: int
    out_token: any
    out_amount: any


def cvxpy_to_data(
    input: Input,
    data: AllData,
    ctx: Ctx,
    result: CvxpySolution,
    ratios=None,
) -> Union[Exchange, Spawn]:
    """_summary_
    Converts Angeris CVXPY result to executable route.
    Receives solution along with all data and context.
    Clean up near zero trades.
    Make `delta-lambda` to be just single trades over venues.
    Start building fork-join supported route tree tracking venue.
      Find starter node and recurse with minus from input matrix (loops covered).
    Visualize.
    """
    if ratios is None:
        ratios = {asset_id: 1 for asset_id in data.all_tokens}

    _etas, trades_raw = parse_total_traded(ctx, result)
    if ctx.debug:
        logger.info("trades_raw", trades_raw)

    # attach tokens ids to trades
    total_trades = []

    for i, raw_trade in enumerate(trades_raw):
        if np.abs(raw_trade[0]) > 0:
            [token_index_a, token_index_b] = data.venues_tokens[i]
            if raw_trade[0] < 0:
                total_trades.append(
                    VenueOperation(
                        in_token=token_index_a,
                        in_amount=-raw_trade[0] / ratios[token_index_a],
                        out_token=token_index_b,
                        out_amount=raw_trade[1] / ratios[token_index_b],
                        venue_index=i,
                    )
                )
            else:
                total_trades.append(
                    VenueOperation(
                        in_token=token_index_b,
                        in_amount=-raw_trade[1] / ratios[token_index_b],
                        out_token=token_index_a,
                        out_amount=raw_trade[0] / ratios[token_index_a],
                        venue_index=i,
                    )
                )
        else:
            total_trades.append(None)

    # # balances
    # in_tokens = defaultdict(int)
    # out_tokens = defaultdict(int)
    # for trade in total_trades:
    #     if trade:
    #         in_tokens[trade.in_token] += trade.in_amount
    #         out_tokens[trade.out_token] += trade.out_amount

    if ctx.debug:
        print(total_trades)
    raise Exception(total_trades)

    # add nodes until burn all input from balance
    # node identity is token and amount input used and depth
    # loops naturally expressed in tree and end with burn
    depth = 0

    def next(current, depth, input):
        print(current)
        depth += 1
        if depth > 10:
            raise Exception("failed to stop ", current)

        # handle big amounts first
        from_coin = sorted(
            [
                trade
                for trade in total_trades
                if trade and trade.in_token == current.name and trade.in_amount > 0 and trade.out_amount > 0
            ],
            key=lambda x: x.in_amount,
            reverse=True,
        )
        burn = current.in_amount
        if burn <= 0:
            return

        nodes = []
        for trade in from_coin:
            trade: VenueOperation = trade
            traded = min(burn, trade.in_amount)
            print(trade)
            print(traded)

            burn -= traded
            trade.in_amount -= traded            
            in_amount= traded * trade.out_amount / trade.in_amount
            if in_amount > 0:
                print("AMOUNT", in_amount)

                next_trade = Node(
                    name=trade.out_token,
                    parent=current,
                    venue_index = trade.venue_index,
                    in_amount = in_amount,
                )
                nodes.append(next_trade)
        for next_trade in nodes:
            next(next_trade, depth, input)
            
    start = Node(name=input.in_token_id, in_amount = input.in_amount, venue_index = -1)
    next(start, depth, input)

    if ctx.debug:
        for pre, _fill, node in RenderTree(start):
            logger.info(
                "%s via=%s in=%s/%s"
                % (
                    pre,
                    node.venue_index,
                    node.in_amount,
                    node.name,
                )
            )
    def next_route(parent_node):
        subs = []
        if parent_node.children:
            for child in parent_node.children:
                sub = next_route(child)
                subs.append(sub)
        venue = data.venue_by_index(parent_node.venue_index)
        if isinstance(venue, AssetPairsXyk):
            return Exchange(
                in_asset_amount=math.ceil(op.in_amount),
                out_amount=math.floor(op.out_amount),
                out_asset_id=op.out_token,
                pool_id=str(venue.pool_id),
                next=subs,
            )
        elif isinstance(venue, AssetTransfers):
            return Spawn(
                in_asset_id=op.in_token,
                in_asset_amount=math.ceil(op.in_amount),
                out_asset_id=op.out_token,
                out_amount=math.floor(op.out_amount),
                next=subs,
            )
        else:
            raise Exception("Unknown venue type")

    return next_route(start_coin)


def parse_total_traded(ctx: Ctx, result: CvxpySolution) -> tuple[any, list]:
    etas = result.eta_values
    deltas = result.delta_values
    lambdas = result.lambda_values

    # clean up near zero trades
    for i in range(result.count):
        if etas[i] < ctx.minimal_amount:
            etas[i] = 0
            deltas[i] = np.zeros(len(deltas[i]))
            lambdas[i] = np.zeros(len(lambdas[i]))
        elif (
            np.max(np.abs(deltas[i])) < ctx.minimal_tradeable_number
            and np.max(np.abs(lambdas[i])) < ctx.minimal_tradeable_number
        ):
            etas[i] = 0
            deltas[i] = np.zeros(len(deltas[i]))
            lambdas[i] = np.zeros(len(lambdas[i]))

    # trading instances
    trades_raw = []
    for i in range(result.count):
        raw_trade = lambdas[i] - deltas[i]
        if np.max(np.abs(raw_trade)) < ctx.minimal_tradeable_number:
            etas[i] = 0
            deltas[i] = np.zeros(len(deltas[i]))
            lambdas[i] = np.zeros(len(lambdas[i]))
        trades_raw.append(lambdas[i] - deltas[i])
    # for i in range(result.count):
    #     if etas[i] >= 1.0 - ctx.minimal_amount:
    #         etas[i] = None
    return etas, trades_raw
