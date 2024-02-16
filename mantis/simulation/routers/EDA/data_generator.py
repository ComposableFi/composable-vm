from simulation.routers.lautaro import conversor, Edge
from simulation.routers.test_lautaro import new_input, simulate_all_to_all_connected_chains_topology, simulate_all_connected_venue
from simulation.routers.data import (
    AllData,
    Input,
    TId,
    Ctx,
    AssetTransfers,
    AssetPairsXyk,
    TAmount,
)
import copy
import pandas as pd
from random import randint
import threading as th

def GetNetwork() -> tuple[Input, AllData]:
    input = new_input("WETH", "ATOM", 2000, 1)
    CENTER_NODE, chains = simulate_all_to_all_connected_chains_topology(input)
    #print(chains)
    all_data = simulate_all_connected_venue(CENTER_NODE, chains)
    return input, all_data

def route_kernel(
    input : Input,
    edges : list[Edge],
    tokensIds : dict[TId, int],
    all_tokens : list[TId],
    max_depth: int = 5,  # The maximum number of edges that can be used
    splits: int = 1000,  # The number of flow units in which the amount is divided
    revision: bool = True,  # When uses an edge, check if the edge has been used before and if so, use the same edge
):

    # If max_depth or splits are not lists, convert them to lists
    if isinstance(max_depth, int):
        max_depth = [max_depth]
    if isinstance(splits, int):
        splits = [splits]

    outcomes: list[float] = [0]
    totSplits = sum(splits)

    # Some constants
    n = len(all_tokens)
    u_init = tokensIds[input.in_token_id]
    u_end = tokensIds[input.out_token_id]

    # The dist and previous edge of each node for each length of the path
    dist = [(None, 0)] * ((max(max_depth) + 1) * n)

    # For each max_depth and splits
    for max_depth_i, splits_i in zip(max_depth, splits):
        for split in range(
            splits_i
        ):  # The split variable is not used but left for clarity
            # Reset the dist and previous edge of each node for each length of the path
            for i in range(len(dist)):
                dist[i] = (None, 0)

            # Initialize the first node
            dist[u_init] = (None, input.in_amount / (totSplits))

            # Actualizate the state
            depth = 0
            # Process each legth of the path
            for step in range(max_depth_i):
                for ei, e in enumerate(edges):
                    for u in e.U:
                        if dist[step * n + u][1] == 0:
                            continue
                        v = e.GetOther(u)
                        if revision:  # If the revision is active, use the same edge if it has been used before
                            ee = copy.deepcopy(e)
                            vv = u
                            # Go back in the path to check if the edge has been used before
                            for jj in range(step, 0, -1):
                                pad = dist[jj * n + vv][0]
                                vv = edges[pad].GetOther(vv)
                                if pad == ei:
                                    ee.DoChange(
                                        vv, dist[(jj - 1) * n + vv][1]
                                    )
                        else:
                            ee = e  # If the revision is not active, use the edge
                        # Get the amount of the other token
                        Xv = ee.GetAmount(u, dist[step * n + u][1])
                        # Update the amount of the other token if it is greater than the previous amount
                        if dist[(step + 1) * n + v][1] < Xv:
                            dist[(step + 1) * n + v] = (ei, Xv)

            # Get the optimal path
            for j in range(1, max_depth_i + 1):
                if dist[j * n + u_end][1] > 0 and (
                    depth == 0
                    or dist[j * n + u_end][1]
                    > dist[depth * n + u_end][1]
                ):
                    depth = j

    #        for i in range(max_depth_i + 1):
    #            print(f"Depth: {i}, Dist: {dist[i * n : (i + 1) * n]}")
    #       if depth == 0:  # if there is no path
    #            raise Exception("No path found")

            path: list[int] = [0] * depth
            v = u_end

            # Rebuild the path
            for j in range(depth, 0, -1):
                path[j - 1] = dist[j * n + v][0]
                v = edges[path[j - 1]].GetOther(v)

            # Use the path and update the edges
            Xi = input.in_amount / (totSplits)
            u = tokensIds[input.in_token_id]
            nods = [u]
            for i in range(len(path)):
            #    print(f"Step: {i}, Nod : {u}, Xi: {Xi}, Theory: {dist[i* n + u][1]}, Edge: {edges[path[i]]}, Amount: {edges[path[i]].GetAmount(u,Xi)}")
                assert abs(Xi - dist[i* n + u][1])<1e-6
                e = edges[path[i]]
                #print(i, u, e.U)
                #if e.GetAmount(u,Xi) < 0:
                #    print(e)
                #    print(u, Xi, e.GetAmount(u,Xi))
                Xj = e.DoChange(u, Xi)
                Xi = Xj
                assert (u==e.U[0] or u==e.U[1])
                u = e.GetOther(u)
                nods.append(u)
                
            # Update the paths and outcomes
            #print(f"Depth: {depth}, Xi: {Xi}, Theory: {dist[depth * n + u_end][1]}, Nods: {nods}")
            assert abs(Xi - dist[depth * n + u_end][1]) < 1e-6
            assert Xi > 0
            outcomes.append(outcomes[-1] + Xi)

    return outcomes[-1]

if __name__ == "__main__": 
    print(pd.__version__)
    input, all_data = GetNetwork()
    edges, tokensIds, all_tokens = conversor(all_data)
    #print(edges)
    #print(len(edges))
    dephts = [3, 5, 10, 15]
    data = {
        "Iters": [],
        "Dzmitry": []
    }
    for d in dephts:
        data[f"Depth {d}"] = []

    def Paso(d : int):
        r = route_kernel(input, copy.deepcopy(edges), tokensIds, all_tokens, splits=1000, max_depth=d)
        data[f"Depth {d}"].append(r)

    for i in range(int(1e4)):
        u_init = randint(0, len(all_tokens)-1)
        u_end = randint(0, len(all_tokens)-1)
        
        while u_end == u_init:
            u_end = randint(0, len(all_tokens)-1)
        
        loc_input = new_input(all_tokens[u_init], all_tokens[u_end], 2000, 1)
        outcomes = route_kernel(loc_input, edges, tokensIds, all_tokens, splits = 10, max_depth = 3, revision=False)
        print(i)
        if (i+1)%1_000 == 0:
            ths = [th.Thread(target=Paso, args=(d,)) for d in dephts]
            for t in ths:
                t.start()
            
            data["Iters"].append(i+1)
            for t in ths:
                t.join()
    df = pd.DataFrame(data)

    df.to_csv("results.csv")