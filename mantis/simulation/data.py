# for alignment on input and output of algorithm
import pandas as pd

from typing import TypeVar

TAssetId = TypeVar("TAssetId")
TNetworkId = TypeVar("TNetworkId")

class AllData:
    simulate_transfers

def simulate_assets_transfers() -> AllData:
    asset_transfers = pd.read_csv("asset_transfers.csv")
    



# port this https://github.com/ComposableFi/xc-solver-rs/blob/main/solver/src/data.rs
class Routes:
    # asset ids and their usd price if available, and for sure their network ids
    # asset globally unique
    # also known as denom in Cosmos or ERC20 token Ethereum or SPL20 in Solanas
    assets = []
    # network ids (chain ids, parachains ids, domains, consensus, whatever it is know)
    networks = []
    # is there is possible to send from network to network, and if possible, what is normalized to used
    # and also asset id and price of gas in native token
    network_to_network = []
    # pools with asset ids they map with reserves, and usd info if available, and swap fee
    # please note that 
    # and network id it is on
    pools = []
    # when asset moved to network, what asset it becomes

    asset_to_network = []
    
    
