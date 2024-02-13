from collections import defaultdict
from copy import copy
from attr import dataclass
from cvm_runtime.execute import Exchange
import cvxpy as cp
import numpy as np
from simulation.routers.data import AllData, AssetPairsXyk, AssetTransfers, Ctx, Input, Spawn
from anytree import Node, RenderTree
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
    
    
    # trading instances
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
    class Trade:
        in_token: any
        in_amount : int
        out_token: any
        out_amount: any
    
    for i, raw_trade in enumerate(trades_raw):
        if np.abs(raw_trade[0]) > 0: 
            [token_index_a, token_index_b] = all_data.venues_tokens[i]
            if raw_trade[0] < 0:                
                trades.append(Trade(in_token=token_index_a, in_amount=-raw_trade[0], out_token=token_index_b, out_amount=raw_trade[1]))
            else:
                trades.append(Trade(in_token=token_index_b, in_amount=-raw_trade[1], out_token=token_index_a, out_amount=raw_trade[0]))
        else: 
            trades.append(None)
    
    # balances
    in_tokens = defaultdict(int)
    out_tokens= defaultdict(int)
    for trade in trades:
        if trade:
            in_tokens[trade.in_token] += trade.in_amount
            out_tokens[trade.out_token] += trade.out_amount
    
    # make deducible mounts by in/out key so can sub it until end
    inouts = {}
    tokens = set()
    start = Node(name=input.in_token_id)
    def next(parent_node):
        for trade in trades:
            if trade:
                if trade[0][0] == parent_node.name:
                    node = Node(name=trade[1][0], parent=parent_node)
                    next(node)
                    tokens.add(trade[1][0])
        
    for trade in trades:
        
        if trade:
            inouts[(trade[0][0],trade[1][0])] = (trade[0][1],trade[1][1])
    
    
# so redesign is to add nodes until gas (amount) is burned. so same node(token id + depth) appears, but with different children until remaining amount is less than some E or same loop traversed already, then just RenderNode to grahiz
# start traversing with biggest input to smalles (sort before traverse)
    
    
    # attach venue ids to trades
    
    # raw_edges = []
    # for i, trade in enumerate(trades):
    #     if trade:
    #         venue = all_data.venue_by_index(i)
    #         if isinstance(trade, AssetTransfers):
    #            raw_edges.push(Spawn()) 
    #         elif isinstance(trade, AssetPairsXyk)
    
    raise Exception((inouts))
    
        
        