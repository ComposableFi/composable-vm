from dataclasses import dataclass
from abc import ABC, abstractmethod

import numpy as np
import cvxpy as cp
from cvxpy.expressions.expression import Expression
from cvxpy.expressions.variable import Variable

from simulation.routers.data import AllData, Input, Output, test_all_data


@dataclass
class RouterAsset:
    id: int
    price: float


@dataclass
class RouterExchange(ABC):
    assets: list[RouterAsset]
    fix_cost: float
    reserves: list[float]
    fee_in: float
    fee_out: float

    @abstractmethod
    def constraints(self, new_reserve: Expression) -> list[Expression]:
        pass


class ConstantProductPool(RouterExchange):
    def constraints(self, new_reserve: Expression) -> list[Expression]:
        return [cp.geo_mean(new_reserve) >= cp.geo_mean(self.reserves)]


class IbcTransfer(RouterExchange):
    def constraints(self, new_reserve: Expression) -> list[Expression]:
        return [cp.sum(new_reserve) >= cp.sum(self.reserves), new_reserve >= 0]


class ConvexRouter():
    """"""
    MICP_THRESHOLD: int = 20    # TODO: this number may need some tuning, 
                                #     to get better accuracy/speed
    """Threshold of exchanges in a problem where using MI (mixed integer) 
    is infeasible and too slow"""


    def parse_data(self, data: AllData) -> tuple[list[RouterExchange], list[RouterAsset]]:
        assets: set[RouterAsset] = set()
        exchanges: list[RouterExchange] = []

        # Pools 
        for pool in data.asset_pairs_xyk:
            if pool.pool_value_in_usd is None:
                # TODO: Maybe just ignore this pools???
                raise ValueError(f"There is no information about USD value for pool {pool}")
            
            token_in_id = pool.in_asset_id
            token_out_id = pool.out_asset_id


            exchanges.append(
                ConstantProductPool(assets=[], 
                                    fix_cost=0, #TODO: No information about this yet 
                                    reserves=[pool.in_token_amount, pool.out_token_amount], 
                                    fee_in=pool.fee_of_in_per_million/1e6, 
                                    fee_out=pool.fee_of_out_per_million/1e6
                    )
            )

        # Transfers
        for transfer in data.asset_transfers:
            # TODO: No way to determine the price per asset when asset is not in 
            # pool, is there any way to get the list of USD price, per asset, to
            # route this fixed price in USD
            IbcTransfer(assets=[], 
                                    fix_cost=transfer.usd_fee_transfer,
                                    reserves=[1e20, 1e20], 
                                    fee_in=0, # No fee, only fix cost at the moment
                                    fee_out=0
                    )
        return exchanges, assets

    def get_new_reserve(
        self, delta: Variable, lamb: Variable, exchange: RouterExchange
    ) -> Expression:
        return exchange.reserves + (1 - exchange.fee_in) * delta - (1 - exchange.fee_out) * lamb

    def solve(self, data: AllData, input: Input) -> Output:
        if not input.max:
            raise NotImplementedError("'max' value on input is not supported to be False yet")
        exchanges, assets = self.parse_data(data)
        return self._solve(exchanges, assets, input)

    def _solve(
        self, exchanges: list[RouterExchange], assets: list[RouterAsset], input: Input
    ) -> Output:
        def asset_index(token_id: int) -> int:
            return [i for i, asset in enumerate(assets) if asset.id == token_id][0]
        
        is_micp = len(exchanges) <= self.MICP_THRESHOLD
        num_exchanges = len(exchanges)
        num_assets = len(assets)

        initial_assets = np.zeros(num_assets)  # Initial assets

        initial_assets[asset_index(input.in_token_id)] = input.in_amount

        A = []
        for exchange in exchanges:
            n_i = len(exchange.assets)  # Number of tokens in pool (default to 2)
            A_i = np.zeros((num_assets, n_i))
            for i, token in enumerate(exchange.assets):
                A_i[asset_index(token.id), i] = 1
            A.append(A_i)

        # tendered (given) amount
        deltas = [
            cp.Variable(len(exchange.assets), nonneg=True) for exchange in exchanges
        ]

        # received (wanted) amounts
        lambdas = [
            cp.Variable(len(exchange.assets), nonneg=True) for exchange in exchanges
        ]
        
        # Binary value, indicates tx or not for given pool
        eta = cp.Variable(
            num_exchanges, boolean=is_micp, nonneg=not is_micp
        )  

        # network trade vector - net amount received over all trades(transfers/exchanges)
        psi = cp.sum(
            [A_i @ (LAMBDA - DELTA) for A_i, DELTA, LAMBDA in zip(A, deltas, lambdas)]
        )

        # Objective is to trade number_of_init_tokens of asset origin_token for a maximum amount of asset objective_token
        obj = cp.Maximize(
            psi[asset_index(input.out_token_id)]
            - cp.sum(eta @ [exchange.fix_cost for exchange in exchanges])
        )

        # Trading function constraints
        constrains = [
            psi + initial_assets >= 0,
        ]

        for i, exchange in enumerate(exchanges):
            new_reserve = self.get_new_reserve(deltas[i], lambdas[i], exchange)
            constrains.extend(exchange.constraints(new_reserve))
            if not is_micp:
                constrains.append(eta >= 0)
                constrains.append(eta <= 15)
                constrains.append(
                    cp.sum(deltas[i] @ [asset.price for asset in exchange.assets])
                    <= eta[i]
                    * 20
                    * input.in_amount
                    * assets[asset_index(input.in_token_id)].price
                )
            else:
                constrains.append(deltas[i] <= eta[i] * exchange.reserves * 0.5)

        # Set up and solve problem
        prob = cp.Problem(obj, constrains)
        prob.solve(solver=cp.SCIP, verbose=False)

        print(
            f"\033[1;91m[{num_exchanges}]Total amount out: {psi.value[asset_index(input.out_token_id)]}\033[0m"
        )

        for i in range(num_exchanges):
            print(
                f"Market {exchanges[i].assets[0]}<->{exchanges[i].assets[1]}, delta: {deltas[i].value}, lambda: {lambdas[i].value}, eta: {eta[i].value}",
            )
        if not is_micp:
            exchanges = [
                pair[0]
                for pair in sorted(
                    zip(exchanges, eta), key=lambda ele: ele[1].value, reverse=True
                )
            ][:self.MICP_THRESHOLD]
            self._solve(exchanges, assets, input)
        else:
            return deltas, lambdas, psi, eta


if __name__ == "__main__":

    data = test_all_data()

    router = ConvexRouter()
    input_obj = Input(in_token_id=1, 
                      out_token_id=4, 
                      in_amount=1, 
                      out_amount=100, 
                      max=True
                      )
    router.solve(data, input_obj)
