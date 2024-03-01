# CVXPY Angeris helper algorithm which more or less independnt of exact implementation


import copy
import math

from anytree import RenderTree
from loguru import logger

from simulation.routers.data import (
    AllData,
    Ctx,
    Exchange,
    Input,
    RouteTree,
    SingleInputAssetCvmRoute,
    Spawn,
)

from .data import CvxpySolution, CvxpyVenue


def cvxpy_to_data(
    input: Input,
    data: AllData,
    ctx: Ctx,
    raw_solutions: list[CvxpySolution],
    ratios: dict = None,
) -> SingleInputAssetCvmRoute:
    """_summary_
    Converts Angeris CVXPY result to executable route.
    Burns over trades from input to final token.
    Here can decide on max split and depth.
    Also can rollback arbitrage loop.
    Compares predicted with routed.
    """

    if not ratios or len(ratios) == 0:
        ratios = {asset_id: 1 for asset_id in data.all_tokens}

    routes = []

    for raw_solution in raw_solutions:
        try:
            venues = [
                CvxpyVenue(
                    i,
                    raw_solution.delta_values[i],
                    raw_solution.lambda_values[i],
                    raw_solution.eta_values[i],
                    venue,
                    ratios,
                )
                for i, venue in enumerate(copy.deepcopy(data.venues))
            ]

            def build_routes(parent, out_asset_amount, out_asset_id, depth):
                if depth > 10:
                    return
                depth += 1
                parent_out_asset_id = parent.trade.out_asset_id
                parent_out_amount = parent.trade.out_asset_amount

                predicted_venues = sorted(
                    [
                        venue
                        for venue in venues
                        if venue.in_coin[0] == parent_out_asset_id
                        and venue.in_coin[1] > parent_out_amount / 10
                        and venue.out_coin[1] > 0
                        and venue.eta == 1.0
                    ],
                    reverse=True,
                    key=lambda x: x.in_coin[1],
                )

                if len(predicted_venues) == 0:
                    logger.error(f"no venues found for {parent_out_asset_id} {parent_out_amount}")
                children = []
                for venue in predicted_venues:
                    in_coin = venue.in_coin
                    traded = min(in_coin[1], parent_out_amount)
                    if traded > 0:
                        received = copy.deepcopy(venue).trade(traded)
                        print
                        logger.error(f"!!!!!!!!!!!!!!!!!!!!!!!!!!! {received} {venue.out_coin[1]}")
                        received = min(received, venue.out_coin[1])
                        if received > 0:
                            if venue.out_coin[0] == out_asset_id:
                                if out_asset_amount / 10 > received:
                                    continue
                                else:
                                    out_asset_amount += received
                            received = venue.trade(traded)
                            parent_out_amount -= traded
                            child = RouteTree(
                                name="transfer" if venue.is_transfer else "exchange",
                                parent=parent,
                                trade=Exchange(
                                    in_asset_id=venue.in_coin[0],
                                    out_asset_id=venue.out_coin[0],
                                    in_asset_amount=int(math.ceil(traded)),
                                    out_asset_amount=int(math.ceil(received)),
                                    pool_id=str(venue.venue.pool_id),
                                    next=[],
                                )
                                if venue.is_exchange
                                else Spawn(
                                    in_asset_id=venue.in_coin[0],
                                    out_asset_id=venue.out_coin[0],
                                    in_asset_amount=int(math.ceil(traded)),
                                    out_asset_amount=int(math.ceil(received)),
                                    next=[],
                                ),
                            )
                            logger.error(f"traded {traded} received {received} {venue.venue}")
                            build_routes(child, out_asset_amount, out_asset_id, depth)
                            children.append(child)
                parent.children = children

            route = RouteTree(
                name="start",
                trade=SingleInputAssetCvmRoute(
                    out_asset_id=input.in_token_id,
                    out_asset_amount=int(math.ceil(input.in_amount)),
                    next=[],
                ),
            )
            build_routes(route, 0, input.out_token_id, 0)
            for pre, _fill, node in RenderTree(route):
                logger.error(f"{pre} {node.name} {node.trade}")

            out_amount = route.ends(input.out_token_id)
            logger.error(f"ends {out_amount}")
            routes.append(route)
        except Exception as e:
            logger.error(f"cvxpy_to_data error {e}")
    assert len(routes) > 0
    return routes
