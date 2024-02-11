from copy import copy
from attr import dataclass
import cvxpy as cp
import numpy as np
from simulation.routers.data import Ctx


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

    def __post_init__(self):
        assert len(self.deltas) > 0
        assert len(self.deltas) == len(self.lambdas) == len(self.eta_values)

    @property
    def count(self):
        return len(self.deltas)
                   
    def received(self, global_index) -> float:
        return self.psi.value[global_index]


def cvxpy_to_data(input, all_data, ctx: Ctx, result: CvxpySolution):
    """_summary_
    Converts Angeris CVXPY result to executable route

    Mangle input into single state into single local index state object.
    Clean up `delta - lambda=0` and ETA=0 zero values.
    Make `delta-lambda` to be just single trades over venues.
    Start building fork-join supported route tree tracking venue.
      Find starter node and recurse with minus from input matrix (loops covered).

    Run over fork-join tree and product no joins (split rotes).

    Generate DOT from tree to visualize.
    """
    result = copy.deepcopy(result)
    
    for i in range(result.count):
        