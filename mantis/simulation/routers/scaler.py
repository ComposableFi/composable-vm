# see https://or.stackexchange.com/questions/11603/numerical-infeasibility-for-moving-numbers-along-some-specific-conversion-edges

from collections import defaultdict
from simulation.routers.data import AllData, Ctx, Input, Output
import  copy

def scale_in(base_data: AllData, input: Input, ctx: Ctx) -> tuple[AllData, Input]:
    """
    Scales in data to be used by simulation
    """
    
    # so we set all transfers amount to some estimate
    for transfer in base_data.asset_transfers:
        transfer.in_token_amount = base_data.maximal_reserves_of(transfer.in_asset_id)
        transfer.out_token_amount = base_data.maximal_reserves_of(transfer.out_asset_id)

    new_data = copy.deepcopy(base_data)
    new_input = copy.deepcopy(input)
    oracalized_data = copy.deepcopy(base_data)
    oracles = base_data.usd_oracles
    all_asset_ids = base_data.all_tokens
        
    oracalized_input = input.in_amount * base_data.token_price_in_usd(input.in_token_id)
    assert oracalized_input > 0
    
    # make all exchanges to be oracalized
    for i, exchange in enumerate(base_data.asset_pairs_xyk):
        oracalized_data.asset_pairs_xyk[i].in_token_amount = exchange.in_token_amount * base_data.token_price_in_usd(exchange.in_asset_id) 
        oracalized_data.asset_pairs_xyk[i].out_token_amount = exchange.out_token_amount * base_data.token_price_in_usd(exchange.out_asset_id) 
    # make transfers oracalized
    for i, transfer in enumerate(base_data.asset_transfers):
        oracalized_data.asset_transfers[i].in_token_amount = transfer.in_token_amount * base_data.token_price_in_usd(transfer.in_asset_id) 
        oracalized_data.asset_transfers[i].out_token_amount= transfer.out_token_amount * base_data.token_price_in_usd(transfer.out_asset_id) 
    for asset_id in oracalized_data.all_tokens:
        oracalized_data.usd_oracles[asset_id] = 1
                        
    maximal_oracalized_reservers = defaultdict()

    
    # cap all big amounts
    for asset_id in all_asset_ids:
        for i, oracalized_venue in enumerate(oracalized_data.asset_pairs_xyk):
            if oracalized_venue.in_asset_id == asset_id:
                oracalized_reserve = oracalized_venue.in_token_amount
                ratio = oracalized_input / oracalized_reserve
                if ratio < ctx.min_input_to_reserve_ratio:
                    new_data.asset_pairs_xyk[i].in_token_amount = new_data.asset_pairs_xyk[i].in_token_amount * (ratio /  ctx.min_input_to_reserve_ratio)
            if oracalized_venue.out_asset_id == asset_id:
                oracalized_reserve = oracalized_venue.out_token_amount
                ratio = oracalized_input / oracalized_reserve
                if ratio < ctx.min_input_to_reserve_ratio:
                    new_data.asset_pairs_xyk[i].out_token_amount = new_data.asset_pairs_xyk[i].out_token_amount * (ratio /  ctx.min_input_to_reserve_ratio)
        for i, oracalized_venue in enumerate(oracalized_data.asset_transfers):
            if oracalized_venue.in_asset_id == asset_id:
                oracalized_reserve = oracalized_venue.in_token_amount
                ratio = oracalized_input / oracalized_reserve
                if ratio < ctx.min_input_to_reserve_ratio:
                    new_data.asset_transfers[i].in_token_amount = new_data.asset_transfers[i].in_token_amount * (ratio /  ctx.min_input_to_reserve_ratio)
            if oracalized_venue.out_asset_id == asset_id:
                oracalized_reserve = oracalized_venue.out_token_amount
                ratio = oracalized_input / oracalized_reserve
                if ratio < ctx.min_input_to_reserve_ratio:
                    new_data.asset_transfers[i].out_token_amount = new_data.asset_transfers[i].out_token_amount * (ratio /  ctx.min_input_to_reserve_ratio)
        
                                    
                    
        # ratio = oracalized_input / oracalized_data.maximal_reserves_of(asset_id)
        # if ratio < ctx.min_input_to_reserve_ratio:
            
        
    return new_data, new_input


def scale_out() -> Output:
    """
    Scales outs
    """
    pass
