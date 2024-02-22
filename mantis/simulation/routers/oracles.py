# oracles which tell price via connections
from typing import Union
from disjoint_set import DisjointSet
from simulation.routers.data import TId

# given set of data about assets, find very approximate ratio of one asset to other
class PartialUsdOracle:   
   def route(partial_oracles: dict[TId, Union[float, None]], transfers: list[tuple[TId, TId]]):
      """
      Very fast one and super sloppy on, 
      considers if there is connection in general,
      same price and all price everywhere.
      No penalty for high fees/long route and non equilibrium.
      """
      ds = DisjointSet()
      for t in transfers:
         ds.union(t[0], t[1])
      for id, value in partial_oracles.items():
         if value is None:
            for other, value in partial_oracles.items():
               if value and ds.connected(id, other):
                  partial_oracles[id] = value
            
                     
def test():
   oracles = {
      1 : 1.0,
      2: None,
      3 : None,
      4 : 2.0,
   }
   transfers =  [
      (2, 4),
      (1, 2),
   ]
   
   PartialUsdOracle.route(oracles, transfers)
   assert oracles[2] == 2.0 
   assert oracles[1] == 1.0 
   assert oracles[3] is None 
   assert oracles[4] == 2.0 