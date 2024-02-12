from copy import copy
from attr import dataclass
from cvm_runtime.execute import Exchange
import cvxpy as cp
import numpy as np
from simulation.routers.data import AllData, AssetPairsXyk, AssetTransfers, Ctx, Input, Spawn


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


def cvxpy_to_data(input: Input, all_data : AllData, ctx: Ctx, result: CvxpySolution):
    """_summary_
    Converts Angeris CVXPY result to executable route.
    
    Receives solution along with all data and context.
    
    Clean up near zero trades.
    
    Make `delta-lambda` to be just single trades over venues.
    Start building fork-join supported route tree tracking venue.
      Find starter node and recurse with minus from input matrix (loops covered).

    Run over fork-join tree and product no joins (split rotes).

    Generate DOT from tree to visualize.
    """
    
    etas = result.eta_values
    deltas = result.delta_values
    lambdas = result.lambda_values
    
    # clean up near zero trades
    for i in range(result.count):
        if etas[i] < ctx.minimal_amount:
            etas[i] = 0
            deltas[i] = np.zeros(len(deltas[i]))
            lambdas[i] = np.zeros(len(lambdas[i]))
        
        if np.max(np.abs(deltas[i])) < ctx.minimal_amount and np.max(np.abs(lambdas[i])) < ctx.minimal_amount:
            etas[i] = 0
            deltas[i] = np.zeros(len(deltas[i]))
            lambdas[i] = np.zeros(len(lambdas[i]))            
    
    trades_raw = []        
    for i in range(result.count):
        raw_trade = lambdas[i] - deltas[i]
        if np.max(np.abs(raw_trade)) < ctx.minimal_amount:
            etas[i] = 0
            deltas[i] = np.zeros(len(deltas[i]))
            lambdas[i] = np.zeros(len(lambdas[i]))
        trades_raw.append(lambdas[i] - deltas[i])
        
    # attach tokens ids to trades
    trades = []
    for i, raw_trade in enumerate(trades_raw):
        if np.abs(raw_trade[0]) > 0: 
            [token_index_a, token_index_b] = all_data.venues_tokens[i]
            if raw_trade[0] < 0:
                input = (token_index_a,-raw_trade[0])
                output = (token_index_b,raw_trade[1])
                trades.append((input,output))
            else:
                input  = (token_index_b,-raw_trade[1])
                output = (token_index_a,raw_trade[0])
                trades.append((input,output))
        else: 
            trades.append(None)
    
    # make deducible mounts by in/out key so can sub it until end
    inouts = {}
    
    for trade in trades:
        if trade:
            inouts[(trade[0][0],trade[1][0])] = (trade[0][1],trade[1][1])
    
    
    # attach venue ids to trades
    
    # raw_edges = []
    # for i, trade in enumerate(trades):
    #     if trade:
    #         venue = all_data.venue_by_index(i)
    #         if isinstance(trade, AssetTransfers):
    #            raw_edges.push(Spawn()) 
    #         elif isinstance(trade, AssetPairsXyk)
    
    raise Exception((inouts))
    
        
        