import copy

import deepdiff as dd

from simulation.routers.data import (
    AllData,
    Ctx,
    Input,
    new_data,
    new_input,
    new_pair,
    new_transfer,
)

from .scaler import oracalize_data, scale_in


def test_oracalize_data():
    pair12 = new_pair(1, 1, 2, 0, 0, 1, 1, 10, 20, 20)
    pair23 = new_pair(2, 2, 3, 0, 0, 1, 1, 10, 10, 10)
    base_data = new_data([pair12, pair23], [])
    base_input = new_input(1, 2, 1, 1)
    oracalized_data, oracalized_input = oracalize_data(base_data, base_input, Ctx())

    oracle = base_data.token_price_in_usd(1)
    assert oracle == 0.25
    assert oracalized_data.token_price_in_usd(2) == 1
    assert oracalized_input.in_amount == 0.25
    assert oracalized_data.asset_pairs_xyk[0].in_token_amount == 5
    assert oracalized_data.asset_pairs_xyk[0].out_token_amount == 5
    assert oracalized_data.asset_pairs_xyk[1].in_token_amount == 2.5
    assert oracalized_data.asset_pairs_xyk[1].out_token_amount == 5

    transfer43 = new_transfer(3, 4, 20, 20, 0, 1)
    base_data = new_data([pair12, pair23], [transfer43])
    base_input = new_input(1, 2, 1, 1)
    oracalized_data, oracalized_input = oracalize_data(base_data, base_input, Ctx())
    assert oracalized_data.token_price_in_usd(4) == 1

    transfer54 = new_transfer(5, 4, 30, 30, 0, 1)
    base_data = new_data([pair12, pair23], [transfer43, transfer54])
    base_input = new_input(1, 2, 1, 1)
    oracalized_data, oracalized_input = oracalize_data(base_data, base_input, Ctx())
    assert oracalized_data.token_price_in_usd(5) == 1


def test_scale_in_no_scale():
    pair12 = new_pair(1, 1, 2, 0, 0, 1, 1, 10, 20, 20)
    pair23 = new_pair(2, 2, 3, 0, 0, 1, 1, 10, 10, 10)
    transfer43 = new_transfer(3, 4, 20, 20, 0, 1)
    transfer54 = new_transfer(5, 4, 30, 30, 0, 1)
    base_data = new_data([pair12, pair23], [transfer43, transfer54])
    base_input = new_input(1, 2, 1, 1)
    for asset_id in base_data.all_tokens:
        assert base_data.maximal_reserves_of(asset_id) > 0
    scaled_data, scaled_input, ratios = scale_in(base_data, base_input, Ctx())

    no_scale = dd.DeepDiff(base_data, scaled_data)
    assert len(no_scale.items()) == 0


def scale_out(data: AllData, input: Input, ratios):
    new_data = copy.deepcopy(data)
    new_input = copy.deepcopy(input)
    for venue in new_data.asset_pairs_xyk:
        venue.in_token_amount = venue.in_token_amount / ratios[venue.in_asset_id]
        venue.out_token_amount = venue.out_token_amount / ratios[venue.out_asset_id]
    for venue in new_data.asset_transfers:
        venue.in_token_amount = venue.in_token_amount / ratios[venue.in_asset_id]
        venue.out_token_amount = venue.out_token_amount / ratios[venue.out_asset_id]
    new_input.in_amount = input.in_amount / ratios[input.in_token_id]
    new_input.out_amount = input.out_amount / ratios[input.out_token_id]
    return new_data, new_input


def test_scale_in_uniform():
    scale = 10**12
    pair12 = new_pair(1, 1, 2, 0, 0, 1, 1, 10 * scale, 20 * scale, 20 * scale)
    pair23 = new_pair(2, 2, 3, 0, 0, 1, 1, 10 * scale, 10 * scale, 10 * scale)
    transfer43 = new_transfer(3, 4, 0, 20 * scale, 20 * scale, 1)
    transfer54 = new_transfer(5, 4, 0, 30 * scale, 30 * scale, 1)
    base_data = new_data([pair12, pair23], [transfer43, transfer54])
    base_input = new_input(1, 2, 1 * scale, 1 * scale)
    for asset_id in base_data.all_tokens:
        assert base_data.maximal_reserves_of(asset_id) > 0
    scaled_data, scaled_input, ratios = scale_in(base_data, base_input, Ctx())

    uniform_scale = dd.DeepDiff(base_data, scaled_data)
    assert len(uniform_scale.items()) == 2

    unscaled_data, unscaled_input = scale_out(scaled_data, scaled_input, ratios)

    assert unscaled_data.all_reserves[0][0] == base_data.all_reserves[0][0]
    assert unscaled_data.all_reserves[3][1] == base_data.all_reserves[3][1]
