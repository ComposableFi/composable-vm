# see https://or.stackexchange.com/questions/11603/numerical-infeasibility-for-moving-numbers-along-some-specific-conversion-edges

from collections import defaultdict
from simulation.routers.data import AllData, Ctx, Input, Output
import  copy

def scale_in(data: AllData, input: Input, ctx: Ctx) -> tuple[AllData, Input]:
    """
    Scales in data to be used by simulation
    """
    new_data = copy.deepcopy(data)
    new_input = copy.deepcopy(input)
    oracles = data.usd_oracles
    for transfer in new_data.asset_transfers:
        transfer.amount_of_in_token = new_data.maximal_reserves_of(transfer.in_asset_id)
        transfer.amount_of_out_token = new_data.maximal_reserves_of(transfer.out_asset_id)

    oracalized_input = input.in_amount * data.token_price_in_usd(input.in_token_id)
    assert oracalized_input > 0
    oracalized_exchanges = []
    for exchange in data.asset_pairs_xyk:
        oracalized_in = exchange.in_token_amount * data.token_price_in_usd(exchange.in_asset_id) 
        oracalized_out = exchange.out_token_amount * data.token_price_in_usd(exchange.out_asset_id) 
        oracalized_exchanges.append((oracalized_in, oracalized_out))
    
    oracalized_transfers = []
    for transfer in data.asset_transfers:
        oracalized_in = transfer.amount_of_in_token * data.token_price_in_usd(transfer.in_asset_id) 
        oracalized_out = transfer.amount_of_out_token * data.token_price_in_usd(transfer.out_asset_id) 
        oracalized_transfers.append((oracalized_in, oracalized_out))    
        
    maximal_oracalized_reservers = defaultdict()
    all_asset_ids = data.all_tokens
    
    
    return new_data, new_input


def scale_out() -> Output:
    """
    Scales outs
    """
    pass
