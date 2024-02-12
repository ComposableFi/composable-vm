from copy import copy
from attr import dataclass
import cvxpy as cp
import numpy as np
from simulation.routers.data import AllData, Ctx, Input


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

    Mangle input into single state into single local index state object.
    Clean up `delta - lambda=0` and ETA=0 zero values.
    Make `delta-lambda` to be just single trades over venues.
    Start building fork-join supported route tree tracking venue.
      Find starter node and recurse with minus from input matrix (loops covered).

    Run over fork-join tree and product no joins (split rotes).

    Generate DOT from tree to visualize.
    """
    
    etas = result.eta_values
    deltas = result.delta_values
    lambdas = result.lambda_values
    for i in range(result.count):
        if etas[i] < ctx.minimal_amount:
            etas[i] = 0
            deltas[i] = np.zeros(len(deltas[i]))
            lambdas[i] = np.zeros(len(lambdas[i]))
        
        if np.max(np.abs(deltas[i])) < ctx.minimal_amount and np.max(np.abs(lambdas[i])) < ctx.minimal_amount:
            etas[i] = 0
            deltas[i] = np.zeros(len(deltas[i]))
            lambdas[i] = np.zeros(len(lambdas[i]))            
    trades = []
        
    for i in range(result.count):
        trade = lambdas[i] - deltas[i]
        if np.max(np.abs(trade)) < ctx.minimal_amount:
            etas[i] = 0
            deltas[i] = np.zeros(len(deltas[i]))
            lambdas[i] = np.zeros(len(lambdas[i]))
        trades.append(lambdas[i] - deltas[i])
        
    raise Exception((trades, etas))
    
        
        