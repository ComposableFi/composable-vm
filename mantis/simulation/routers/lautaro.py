# solves using NLP optimization (or what best underlying engine decides)
# Models cross chain transfers as fees as """pools"""
# Uses decision variables to decide if to do Transfer to tap pool or not.

import numpy as np
import cvxpy as cp
import copy 
import threading as th
import os
MAX_RESERVE = 1e10

from simulation.routers.data import AllData, Input, TId, TNetworkId, Ctx, AssetTransfers, AssetPairsXyk, TAmount

class Edge:
    U : list[int]
    B : list[TAmount]
    W : list[int]
    F : list[float]

    def toFloat(self, x):
        try:
            return float(x)
        except:
            return 0.0

    def __init__(self, e : [AssetTransfers|AssetPairsXyk], tokensIds : dict[TId, int]): 
        if isinstance(e,AssetTransfers):
            self.__initFromTransfers(e, tokensIds)
        else:
            self.__initFromPairsXyk(e, tokensIds)

    def __initFromTransfers(self, e : AssetTransfers, tokensIds : dict[TId, int]):
        self.U = [tokensIds[e.in_asset_id], tokensIds[e.out_asset_id]]
        self.B = [e.amount_of_in_token, e.amount_of_out_token]
        self.W = [1,1]
        self.F = [float(e.fee_per_million)/1_000_000.0, float(e.fee_per_million)/1_000_000.0]
    
    def __initFromPairsXyk(self, e : AssetPairsXyk, tokensIds : dict[TId, int]):
        self.U = [tokensIds[e.in_asset_id], tokensIds[e.out_asset_id]]
        self.B = [e.in_token_amount, e.out_token_amount]
        self.W = [e.weight_of_a, e.weight_of_b]
        self.F = [self.toFloat(e.fee_in), self.toFloat(e.fee_out)]

    def GetAmount(self, Ti, Xi):
        i,o = 0,1
        if Ti == self.U[1]:
            i,o = 1,0
        Xi = Xi * (1-self.F[i])
        return self.B[o] * (1-(self.B[i]/(self.B[i]+Xi))**(self.W[i]/self.W[o]))

    def DoChange(self, Ti, Xi):
        i,o = 0,1
        if Ti == self.U[1]:
            i,o = 1,0
        Xi = Xi * (1-self.F[i])
        result = Xi * (self.B[i]/(self.B[i]+Xi))**(self.W[i]/self.W[o])
        self.B[i] += Xi
        self.B[o] -= result        
        return result
    
    def GetOther(self, Ti):
        if Ti == self.U[0]:
            return self.U[1]
        return self.U[0]

class Estado:
    max_depth : int
    depth : int
    dist : list[list[tuple[int, float]]]
    dlock : list[th.Lock]
    u_end : int
    edges : list[Edge]
    revision : bool
    Nopts : int
    j : int
    def __init__(self):
        self.dist = None
        self.max_depth = None
        self.depth = None
        self.u_end = None
        self.edges = None
        self.revision = None
        self.dlock = None
        self.Nopts = None
        self.j = 0
    
def Rango(e0, e1, estado):
        edges = estado.edges
        j = estado.j
        #print(e0, e1, j)

        dist = estado.dist
        for ei in range(e0,e1):
            e = edges[ei]
            for u in e.U:
                if dist[j][u][1] == 0: continue
                v = e.GetOther(u)
                if estado.revision:
                    ee = copy.deepcopy(e)
                    vv = u
                    for jj in range(j,0,-1):
                        pad = dist[jj][vv][0]
                        vv = edges[pad].GetOther(vv)
                        if pad == ei: 
                            ee.DoChange(vv,dist[jj-1][vv][1])
                else : ee = e
                Xv = ee.GetAmount(u, dist[j][u][1])
                estado.dlock[v].acquire()
                if dist[j+1][v][1] < Xv:
                    dist[j+1][v] = (ei, Xv )
                estado.dlock[v].release()

# Bellman Ford based solution

def route(
    input: Input,
    all_data: AllData,
    _ctx: Ctx = Ctx(),
    max_depth:int = 5,
    splits:int = 1000,
    revision = True,
    Nproces = None,
):
    edges : list[Edge] = []
    all_tokens = all_data.all_tokens
    tokensIds = {x:i for i,x in enumerate(all_tokens)}
    
    if Nproces == None: Nproces = os.cpu_count()

    if isinstance(max_depth,int): max_depth = [max_depth]
    if isinstance(splits,int): splits = [splits]

    for x in all_data.asset_transfers:
        edges.append(Edge(x, tokensIds))
    for x in all_data.asset_pairs_xyk:
        edges.append(Edge(x, tokensIds))
    
    n = len(all_tokens)
    
    deltas : list[float] = [0]*len(edges)
    lambdas: list[float] = [0]*len(edges) 
    paths : list[list[int]] = []
    outcomes : list[float] = [0]
    totSplits = sum(splits)

    u_init = tokensIds[input.in_token_id]
    u_end = tokensIds[input.out_token_id]

    estado = Estado()
    estado.u_end = u_end
    estado.edges = edges
    estado.revision = revision
    estado.dlock = [th.Lock() for i in range(n)]
    #print(estado.dlock)

    e0 = [i*len(edges)//Nproces for i in range(Nproces)]
    e1 = [(i+1)*len(edges)//Nproces for i in range(Nproces)]
    e1[-1] = len(edges)
    estado.dlock = [th.Lock() for i in range(n)]
    
    n0 = [i*n//Nproces for i in range(Nproces)]
    n1 = [(i+1)*n//Nproces for i in range(Nproces)]
    n1[-1] = n


    for max_depth_i, splits_i in zip(max_depth, splits):
        for split in range(splits_i):
            dist = [[(None,0) for i in range(n)] for j in range(max_depth_i+1)]
            dist[0][u_init] = (None, input.in_amount/(totSplits))
            estado.depth = 0
            estado.dist = dist
            estado.max_depth = max_depth_i
            
            for step in range(max_depth_i):
                estado.j = step
                threads = [th.Thread(target=Rango, args=(e0[i], e1[i], estado)) for i in range(Nproces)]
                for t in threads: t.start()
                for t in threads: t.join()
            
    #        for f in estado.dist:
    #            print(f)

            for j in range(1,max_depth_i+1):
                if dist[j][u_end] and (estado.depth == 0 or dist[j][u_end][1] > dist[estado.depth][u_end][1]):
                    estado.depth = j

            path : list[int] = [0]*estado.depth
            v = u_end
            
            for j in range(estado.depth, 0, -1):
                path[j-1] = dist[j][v][0]
                v = edges[path[j-1]].GetOther(v)

            Xi = input.in_amount/(totSplits)
            u = tokensIds[input.in_token_id]
            for i in range(len(path)-1):
                e = edges[path[i]]
                deltas[path[i]] += Xi 
                Xj = e.DoChange(u, Xi)
                lambdas[path[i]] += Xj
                Xi = Xj
                v = e.GetOther(u)
            
            paths.append(path)
            outcomes.append(outcomes[-1]+Xi)
    #print(paths)
    return outcomes[-1], outcomes[-2] 
    #return outcome, paths, lambdas, deltas

def BuildRoute(max_depth, splits, revision):
    def _route(input: Input, all_data: AllData, _ctx: Ctx = Ctx()):
        return route(input, all_data, _ctx, max_depth, splits, revision)
    return _route