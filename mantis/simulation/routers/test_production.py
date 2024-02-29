from mantis.simulation.routers.angeris_cvxpy import cvxpy_to_data
from mantis.simulation.routers.generic_linear import route
from simulation.routers.data import Ctx, new_data, new_input, new_pair


def test_1():
    pool_2376844893874674201515870126122 = new_pair(
        2376844893874674201515870126122,
        237684487542793012780631851015,
        237684487542793012780631851016,
        10,
        10,
        1,
        1,
        4192544,
        1000000,
        1000000,
    )
    pool_2376844893874674201515870126124 = new_pair(
        2376844893874674201515870126124,
        237684487542793012780631851010,
        237684487542793012780631851015,
        10,
        10,
        1,
        1,
        271173,
        151671,
        42549,
    )

    data = new_data([pool_2376844893874674201515870126122, pool_2376844893874674201515870126124], [])
    ctx = Ctx()
    input = new_input(237684487542793012780631851015, 237684487542793012780631851016, 446, 1)
    result = route(input, data)
    cvxpy_to_data(input, data, ctx, result)
