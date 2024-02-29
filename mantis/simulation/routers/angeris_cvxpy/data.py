import math
from typing import Union

import cvxpy as cp
import numpy as np
from anytree import Node, NodeMixin, RenderTree
from attr import dataclass
from loguru import logger

from simulation.routers.data import (
    AllData,
    AssetPairsXyk,
    AssetTransfers,
    Ctx,
    Exchange,
    Input,
    Spawn,
)


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
        assert len(self.deltas) == len(self.lambdas)
        assert len(self.deltas) == len(self.eta_values)
        return len(self.deltas)

    def received(self, global_index) -> float:
        return self.psi.value[global_index]
    
class VenuesSnapshot(NodeMixin):
    """_summary_
    The total amount which goes in/out each venue
    """

    venue_index: int
    in_asset_id: any
    in_amount: int
    out_asset_id: any
    out_amount: any

    def __init__(
        self,
        name,
        venue_index,
        in_asset_id,
        in_amount,
        out_asset_id,
        out_amount,
        parent=None,
        children=None,
    ):
        self.name = name
        self.venue_index = venue_index
        self.in_asset_id = in_asset_id
        self.in_amount = in_amount
        self.out_asset_id = out_asset_id
        self.out_amount = out_amount
        self.parent = parent
        if children:
            self.children = children
            
    def __repr__(self):
        return f"{self.name} {self.venue_index} {self.in_amount}/{self.in_asset_id} {self.out_amount}/{self.out_asset_id} {len(self.children)}"