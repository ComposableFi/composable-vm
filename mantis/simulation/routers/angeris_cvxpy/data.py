import math
from typing import Union

import cvxpy as cp
import numpy as np
from attr import dataclass
from loguru import logger

from simulation.routers.data import AllData, AssetPairsXyk, AssetTransfers, Ctx, Input


class ProblemException(Exception):
    pass


class CvxpyVenue:
    index: int
    deltas: list[int]
    lambdas: list[int]
    eta: bool
    venue: Union[AssetPairsXyk, AssetTransfers]
    left: bool

    def __init__(self, index, deltas, lambdas, eta, venue, ratios):
        if deltas[0] > lambdas[0]:
            self.left = True
        else:
            assert deltas[1] >= lambdas[1]
            self.left = False
        self.index = index
        self.deltas = [
            int(math.ceil(deltas[0] / ratios[venue.in_asset_id])),
            int(math.ceil(deltas[1] / ratios[venue.out_asset_id])),
        ]
        self.lambdas = [
            int(math.ceil(lambdas[0] / ratios[venue.in_asset_id])),
            int(math.ceil(lambdas[1] / ratios[venue.out_asset_id])),
        ]
        self.eta = int(eta)
        self.venue = venue
        if self.eta == 1.0:
            logger.error(f"{index} {deltas} {lambdas} {eta} ")
            logger.error(f"{self.index} {self.deltas} {self.lambdas} {self.eta} ")

    def trade(self, traded):
        logger.error(f"trading {traded} {self.venue} {self.index}")
        in_coin = self.in_coin
        assert traded <= in_coin[1]
        self.deltas[0] -= traded
        self.deltas[1] -= traded
        if self.left:
            assert self.deltas[0] >= 0
        else:
            assert self.deltas[1] >= 0
        received = self.venue.trade(in_coin[0], traded)
        assert received > 0
        if self.left:
            assert self.lambdas[1] > 0
        else:
            assert self.lambdas[0] > 0
        # received = min(received, self.lambdas[1] if self.left else self.lambdas[0])

        self.lambdas[0] -= received
        self.lambdas[1] -= received

        return received

    @property
    def is_transfer(self):
        return isinstance(self.venue, AssetTransfers)

    @property
    def is_exchange(self):
        return isinstance(self.venue, AssetPairsXyk)

    @property
    def in_coin(self):
        if self.left:
            return (self.venue.in_asset_id, self.deltas[0])
        else:
            return (self.venue.out_asset_id, self.deltas[1])

    @property
    def out_coin(self):
        if self.left:
            return (self.venue.out_asset_id, self.lambdas[1])
        else:
            return (self.venue.in_asset_id, self.lambdas[0])

    def __post_init__(self):
        assert self.in_coin[0] != self.out_coin[0]


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

    input: Input
    data: AllData
    psi: cp.Variable
    etas: cp.Variable
    problem: cp.Problem

    eta_values: np.ndarray[float]

    def __init__(self, deltas, lambdas, psi, etas, problem, eta_values, input, data):
        self.deltas = deltas
        self.lambdas = lambdas
        self.psi = psi
        self.etas = etas
        self.problem = problem
        self.eta_values = eta_values
        self.input = input
        self.data = data

    @property
    def psi_values(self) -> np.ndarray[float]:
        return np.array([x.value for x in self.psi])

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

    @property
    def predicted_out_amount(self):
        return self.psi.value[self.data.index_of_token(self.input.out_token_id)]

    @property
    def predicted_in_amount(self):
        return self.psi.value[self.data.index_of_token(self.input.in_token_id)]

    @staticmethod
    def ensure_eta(forced_etas):
        if all([eta == 0 for eta in forced_etas]):
            raise Exception(f"all etas are zero, so you cannot trade at all {forced_etas}")

    def to_forced_max(self, all_data, ctx: Ctx):
        """_summary_
        Given trade from draft solution, find maximal possible
        1. max reserve
        2. traded multiplier
        3. input oracle multiplier
        take min of these
        """
        self.cut_unconditional()
        self.cut_small_numbers()
        forced_max = [None] * len(self.delta_values)
        for i, eta in enumerate(self.forced_etas):
            if eta == 0.0:
                forced_max[i] = np.array([0.0, 0.0])
            else:
                forced_max[i] = np.array([0.0, 0.0])
                max_a_asset_reserve = all_data.maximal_reserves_of(all_data.venues_tokens[i][0])
                max_b_asset_reserve = all_data.maximal_reserves_of(all_data.venues_tokens[i][1])
                a = max(self.delta_values[i][0], self.lambda_values[i][0])
                b = max(self.delta_values[i][1], self.lambda_values[i][1])
                forced_max[i][0] = min(
                    ctx.maximal_trading_mistake_factor * ctx.maximal_oracle_mistake_factor * a,
                    max_a_asset_reserve,
                )
                forced_max[i][1] = min(
                    ctx.maximal_trading_mistake_factor * ctx.maximal_oracle_mistake_factor * b,
                    max_b_asset_reserve,
                )

        return forced_max

    @property
    def used_venues(self):
        return sum(self.eta_values)

    @property
    def forced_etas(self):
        self.cut_unconditional()
        self.cut_small_numbers()

        result = [None] * len(self.eta_values)
        for i, eta in enumerate(self.eta_values):
            if eta == 0.0:
                result[i] = eta

        # if there is one and only one route to and from token, make that path 1.0 all the way
        if self.used_venues == 1:
            for i, eta in enumerate(self.eta_values):
                if eta == 1.0:
                    result[i] = 1.0

        return result

    def cut_unconditional(self):
        """
        cuts something which must be cut uncoditionally
        """
        self.verify_core()
        for i, eta in enumerate(self.eta_values):
            if self.get_zeros_count(i) >= 3:
                self.eta_values[i] = 0
            if self.delta_values[i][0] < 0 and self.delta_values[i][1] < 0:
                self.eta_values[i] = 0

    def get_zeros_count(self, i):
        return sum(
            [
                int(np.abs(self.delta_values[i][0]) == 0.0),
                int(np.abs(self.delta_values[i][1]) == 0.0),
                int(np.abs(self.lambda_values[i][0]) == 0.0),
                int(np.abs(self.lambda_values[i][1]) == 0.0),
            ]
        )

    def cut_small_numbers(self):
        """
        If numbers are small and oracle aligns that price is way smaller than input, eliminate
        """
        self.verify_core()
        for i, eta in enumerate(self.eta_values):
            if self.get_zeros_count(i) < 3:
                if self.get_zeros_count(i) >= 2:
                    if (
                        sum(
                            [
                                np.abs(self.delta_values[i][0]),
                                np.abs(self.delta_values[i][1]),
                                np.abs(self.lambda_values[i][0]),
                                np.abs(self.lambda_values[i][1]),
                            ]
                        )
                        <= 1e-10
                    ):
                        logger.info(f"eliminations:: two zeros and two small values for {i}")
                        self.eta_values[i] = 0
                    if (
                        sum(
                            [
                                int(self.delta_values[i][0] < 0.0),
                                int(self.delta_values[i][1] < 0.0),
                                int(self.lambda_values[i][0] < 0.0),
                                int(self.lambda_values[i][1] < 0.0),
                            ]
                        )
                        == 2
                    ):
                        logger.info(f"eliminations:: two zeros and two negatives for {i}")
                        self.eta_values[i] = 0

    def cut_using_oracles(self):
        self.verify_core()
        for i, eta in enumerate(self.eta_values):
            if eta > 0.0:
                venue = self.data.venue_by_index(i)
                price_of_a = self.data.token_price_in_usd(venue.in_asset_id)
                price_of_b = self.data.token_price_in_usd(venue.out_asset_id)
                if price_of_a and price_of_b is not None:
                    if price_of_a > 0 or price_of_b > 0:
                        assert (
                            price_of_a > 0 and price_of_b > 0
                        )  # so oracle must know price of one token if its counter is oracalized
                        d0u = np.abs(self.delta_values[i][0] * price_of_a)
                        d0u = np.abs(self.delta_values[i][1] * price_of_a)
                        l0u = np.abs(self.lambda_values[i][0] * price_of_b)
                        l0u = np.abs(self.lambda_values[i][1] * price_of_b)
                        trade_usd = sum([d0u, d0u, l0u, l0u])
                        if trade_usd <= 0.000001:
                            logger.warning(
                                f"eliminated using sum of values in oracle via {i} in {trade_usd} USD with prices {self.delta_values[i][0]}*{price_of_a} and {self.delta_values[i][1]}*{price_of_b}"
                            )
                            # raise Exception(price_of_a, " ", price_of_b, " ", self.delta_values[i], " ", self.lambda_values[i], " ", i)
                            self.eta_values[i] = 0
                else:
                    logger.error('Operator ">" not supported for "None".')
        pass

    def ensure_bug_trades_pay_fee(self):
        for i, eta in enumerate(self.eta_values):
            venue = self.data.venue_by_index(i)
            price_of_a = self.data.token_price_in_usd(venue.in_asset_id)
            price_of_b = self.data.token_price_in_usd(venue.out_asset_id)
            delta = self.delta_values[i]
            a = delta[0] * price_of_a
            b = delta[1] * price_of_b
            p = self.input.in_amount * self.data.token_price_in_usd(self.input.in_token_id)
            if a > p / 1000 or b > p / 1000:
                logger.warning("was set to zero while traded")
                self.eta_values[i] = 1

    def verify(self, ctx):
        """
        assert assumption for solution on top being feacible
        """
        self.verify_core()
        self.summary(ctx)
        for i, eta in enumerate(self.eta_values):
            if eta == 1.0:
                if not (self.delta_values[i][0] > 0 or self.delta_values[i][1] > 0):
                    raise Exception(i, self.delta_values[i])
        CvxpySolution.ensure_eta(self.eta_values)

    def summary(self, ctx):
        logger.info(
            f"\033[1;91m raw_in={self.psi.value[self.data.index_of_token(self.input.in_token_id)]}->raw_out={self.psi.value[self.data.index_of_token(self.input.out_token_id)]}(({self.psi.value[self.data.index_of_token(self.input.out_token_id)] * self.data.token_price_in_usd(self.input.out_token_id)}/USD))\033[0m"
        )
        logger.info(
            f"original_in={self.input.in_amount}/{self.input.in_token_id}({self.input.in_amount * self.data.token_price_in_usd(self.input.in_token_id)}/USD),original_out={self.input.out_amount}/{self.input.out_token_id}"
        )

        for i in range(self.data.venues_count):
            logger.info(
                f"{i}*({self.eta_values[i]})({self.data.all_reserves[i][0]}/{self.data.assets_for_venue(i)[0]}<->{self.data.all_reserves[i][1]}/{self.data.assets_for_venue(i)[1]}),delta={self.deltas[i].value},lambda={self.lambdas[i].value}",
            )

    def verify_core(self):
        if self.problem.status == "optimal":
            logger.trace(f"Problem status: {self.problem.status}")
        elif self.problem.status == "optimal_inaccurate":
            logger.warning(f"Problem status: {self.problem.status}")
        else:
            raise ProblemException(f"Problem status: {self.problem.status}")

    def received(self, global_index) -> float:
        return self.psi.value[global_index]
