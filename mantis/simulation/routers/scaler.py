# see https://or.stackexchange.com/questions/11603/numerical-infeasibility-for-moving-numbers-along-some-specific-conversion-edges

import copy

from simulation.routers.data import AllData, Ctx, Input


def oracalize_data(base_data: AllData, input: Input, ctx: Ctx) -> tuple[AllData, Input]:
    """
    Basically makes all token values in whole data graph equal in value
    """
    assert base_data is not None
    oracalized_data = copy.deepcopy(base_data)
    oracalized_input = copy.deepcopy(input)
    assert input.in_amount > 0
    oracalized_input.in_amount = input.in_amount * base_data.token_price_in_usd(
        input.in_token_id
    )
    
    if oracalized_input.in_amount < ctx.min_input_in_usd :
        raise Exception(f"minimal amount is {ctx.minimal_amount} and you have {oracalized_input.in_amount} for {input.in_token_id}")

    # make all exchanges to be oracalized
    for i, exchange in enumerate(base_data.asset_pairs_xyk):
        oracalized_data.asset_pairs_xyk[i].in_token_amount = (
            exchange.in_token_amount
            * base_data.token_price_in_usd(exchange.in_asset_id)
        )
        oracalized_data.asset_pairs_xyk[i].out_token_amount = (
            exchange.out_token_amount
            * base_data.token_price_in_usd(exchange.out_asset_id)
        )
    # make transfers oracalized
    for i, transfer in enumerate(base_data.asset_transfers):
        oracalized_data.asset_transfers[i].in_token_amount = (
            transfer.in_token_amount
            * base_data.token_price_in_usd(transfer.in_asset_id)
        )
        oracalized_data.asset_transfers[i].out_token_amount = (
            transfer.out_token_amount
            * base_data.token_price_in_usd(transfer.out_asset_id)
        )
    if oracalized_data.usd_oracles is None:
        oracalized_data.usd_oracles = {}
    for asset_id in oracalized_data.all_tokens:
        oracalized_data.usd_oracles[asset_id] = 1

    return oracalized_data, oracalized_input


def scale_in(
    base_data: AllData, input: Input, ctx: Ctx
) -> tuple[AllData, Input, dict[any, float]]:
    """
    Scales in data to be used by simulation
    """

    # so we set all transfers amount to some estimate
    for transfer in base_data.asset_transfers:
        transfer.in_token_amount = base_data.maximal_reserves_of(transfer.in_asset_id)
        transfer.out_token_amount = base_data.maximal_reserves_of(transfer.out_asset_id)

    new_data = copy.deepcopy(base_data)
    new_input = copy.deepcopy(input)

    oracalized_data, oracalized_input = oracalize_data(base_data, input, ctx)

    all_asset_ids = base_data.all_tokens

    input_token_price = base_data.token_price_in_usd(input.in_token_id)
    input_token_amount = input.in_amount
    input_token_price * input_token_amount

    # cap all big amounts and remove venues which will not give big amount
    for asset_id in all_asset_ids:
        for i, oracalized_venue in enumerate(oracalized_data.asset_pairs_xyk):
            if oracalized_venue.in_asset_id == asset_id:
                oracalized_reserve = oracalized_venue.in_token_amount
                if oracalized_reserve == 0:
                    new_data.asset_pairs_xyk[i].zero()
                else:
                    ratio = oracalized_input.in_amount / oracalized_reserve
                    if ratio < ctx.min_input_to_reserve_ratio:
                        new_data.asset_pairs_xyk[i].in_token_amount = (
                            new_data.asset_pairs_xyk[i].in_token_amount
                            * (ratio / ctx.min_input_to_reserve_ratio)
                        )
                    if oracalized_reserve < ctx.min_usd_reserve:
                        new_data.asset_pairs_xyk[i].zero()
            if oracalized_venue.out_asset_id == asset_id:
                oracalized_reserve = oracalized_venue.out_token_amount
                if oracalized_reserve == 0:
                    new_data.asset_pairs_xyk[i].zero()
                else:
                    ratio = oracalized_input.in_amount / oracalized_reserve
                    if ratio < ctx.min_input_to_reserve_ratio:
                        new_data.asset_pairs_xyk[i].out_token_amount = (
                            new_data.asset_pairs_xyk[i].out_token_amount
                            * (ratio / ctx.min_input_to_reserve_ratio)
                        )
                    if oracalized_reserve < ctx.min_usd_reserve:
                        new_data.asset_pairs_xyk[i].zero()

        for i, oracalized_venue in enumerate(oracalized_data.asset_transfers):
            if oracalized_venue.in_asset_id == asset_id:
                oracalized_reserve = oracalized_venue.in_token_amount
                if oracalized_reserve == 0:
                    new_data.asset_transfers[i].zero()
                else:
                    ratio = oracalized_input.in_amount / oracalized_reserve
                    if ratio < ctx.min_input_to_reserve_ratio:
                        new_data.asset_transfers[i].in_token_amount = (
                            new_data.asset_transfers[i].in_token_amount
                            * (ratio / ctx.min_input_to_reserve_ratio)
                        )
                    if oracalized_reserve < ctx.min_usd_reserve:
                        new_data.asset_transfers[i].zero()
            if oracalized_venue.out_asset_id == asset_id:
                oracalized_reserve = oracalized_venue.out_token_amount
                if oracalized_reserve == 0:
                    new_data.asset_transfers[i].zero()
                else:
                    ratio = oracalized_input.in_amount / oracalized_reserve
                    if ratio < ctx.min_input_to_reserve_ratio:
                        new_data.asset_transfers[i].out_token_amount = (
                            new_data.asset_transfers[i].out_token_amount
                            * (ratio / ctx.min_input_to_reserve_ratio)
                        )
                    if oracalized_reserve < ctx.min_usd_reserve:
                        new_data.asset_transfers[i].zero()

    # zoom into
    ratios = {asset_id: 1 for asset_id in all_asset_ids}
    for asset_id in all_asset_ids:
        maximal_reserve = new_data.maximal_reserves_of(asset_id)
        if maximal_reserve > ctx.max_reserve:
            ratio = ctx.max_reserve / maximal_reserve
            ratios[asset_id] = ratio
            print(ratios[asset_id])
            for venue in new_data.asset_pairs_xyk:
                if venue.in_asset_id == asset_id:
                    venue.in_token_amount = venue.in_token_amount * ratio
                if venue.out_asset_id == asset_id:
                    venue.out_token_amount = venue.out_token_amount * ratio
                if (
                    venue.in_token_amount < ctx.minimal_amount
                    or venue.out_token_amount < ctx.minimal_amount
                ):
                    venue.in_token_amount = 0
                    venue.out_token_amount = 0
            for transfer in new_data.asset_transfers:
                if transfer.in_asset_id == asset_id:
                    transfer.in_token_amount = transfer.in_token_amount * ratio
                if transfer.out_asset_id == asset_id:
                    transfer.out_token_amount = transfer.out_token_amount * ratio
                if (
                    venue.in_token_amount < ctx.minimal_amount
                    or venue.out_token_amount < ctx.minimal_amount
                ):
                    venue.in_token_amount = 0
                    venue.out_token_amount = 0
            if input.in_token_id == asset_id:
                new_input.in_amount = new_input.in_amount * ratio
            if input.out_token_id == asset_id:
                new_input.out_amount = new_input.out_amount * ratio
        # and also we scale up some reserves if token is valuable
        # but scale up until max_reserve

        # here we can clean up small venues, numerically small
    return new_data, new_input, ratios
