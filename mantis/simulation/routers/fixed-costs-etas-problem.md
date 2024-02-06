# Fixed costs (`eta`) problem

Devised [Optimal Routing for Constant Function Market Makers](https://arxiv.org/abs/2204.05238) approach to find optimal routing to (swap) assets.

`5 Fixed transaction costs` introduced ETA variable which is not easy to solve around and proposed that it can decide go/no go variable (boolean, 0 or 1) to decide to tap into pool or not.

For us it is important decision as we need to decide to bridge token or not for solutions.

When problem solved using linear algebra, that leads to high compute costs, and because of limits of hardware numbers to infeasibility.

Several way to solver problem were identified:

1. ETA as integer, and solver engine will handle that. Using big parallel machines and quad precision engines can help with that.
2. ETA to be non integer. In this case solver engine will zero eta for venues it wants. Pick limited set of transfers(and pools behind) using venues with zero eta, and repeat 1.
3. Model problem as non linear explicitly, by multiplying several variables in constraints.
4. After getting ETAs telling what solver engine wants to go, have heuristic to 
1. 
1. 
1. 



    psi = cp.sum(
        [
            A_i @ (LAMBDA - DELTA)
            for A_i, DELTA, LAMBDA in zip(mapping_matrices, deltas, lambdas)
        ]
    )

    # Creating the objective
    objective = cp.Maximize(
        psi[all_tokens.index(obj_token)]
        - cp.sum([eta[i] * all_tx_cost[i] for i in range(len(all_cfmms))])
    )

    # Creating constraints
    constraints = [psi + current_assets >= 0]

def solve_with_known_eta(
    """In this function we solve the optimal routing problem with known eta values. These values are either 0 or 1.
    How we approach this is by not including any CFMMs into the constraints that have an
    eta value equal to 0. This way, we can solve the problem with CVXPY and not run into numerical issues.
   
    # Forcing delta and lambda to be 0 if eta is 0
    for i, cfmm in enumerate(all_cfmms):
        if eta[i] == 0:
            constraints.append(deltas[i] == 0)
            constraints.append(lambdas[i] == 0)
        else:
            constraints.append(deltas[i] <= MAX_RESERVE)



> Rafa Serrano:
I may had the name wrong the name is not MILP it is MINLP it is always non linear... Unless its all IBC transfers, in that case the problem is LP. 
The problem is detected as MI as long as some of the variables are declared as either binary or integer in cvxpy, binary=True integer=True
The idea is having two runs, one being normal convex (very fast inaccurate) other run with filtered pools mixed integer (slow and accurate) 

So the approach is, we need to discriminate some pools in the first calculation, we do it by assigning the weight of the trade done by each pool by the ETA value in the first pass. We then encounter which 20 first pools are the most contributing to the routing. And we rerun the algorithm now with eta being an integer variable. So the meaning of eta is different when the problem is "normal" or "mixed integer". Maybe rewriting the whole thing twice could have been more comprensible

> Rafa Serrano:
One note, when we say MI we are referring only to eta, not deltas and gammas (inputs and outputs of pools) that would be completely infeasible thing to do. As there would not be any way to do any of the convex optimization tricks

> dzmitry lahoda:
Nice in general. May  could be good next time. For now I will go without ETA + route simulator.

> dzmitry lahoda:
ETA has issues with scaling to, do not want to solver it now.

> dzmitry lahoda:
Also I think of next. As soon as problem is NLP, I can multiply variables. If I can multiply 2 variables, I can make ETA to be 2 variables and put these around 0 and around 1. So can solve ETA without MI overall. I can be fully wrong so.

> Rafa Serrano:
I did try this approach, but I think that the solvers don't take kindly that you are trying to cheat math



    # find best etas ranged by etas found on first run (go/no go decision)
    for eta_pivot in sorted(initial_etas):
        try:
            new_deltas, _new_lambdas, new_psi, new_etas = solve(
                all_data,
                input,
                ctx,
                [1 if eta <= eta_pivot else 0 for eta in initial_etas],
            )
            if new_psi.value[all_data.index_of_token(input.out_token_id)] >= received:
                etas = new_etas
                deltas = new_deltas
        except Exception as e:
            print(e)
        
    if ctx.debug:        
        print("eliminating small venues")
    eta_changed = True
    while eta_changed:
        eta_changed = False
        try:
            print("SADSADASDSADSADSDASSA")
            for i, delta_locals in enumerate(deltas):
                if all(delta_i.value < 1e-04 for delta_i in delta_locals):
                    # if input into venue is small, disable using it
                    etas[i] = 0
                    eta_changed = True
                    if ctx.debug:
                        print(f"etas changed so it is {etas}")
            deltas, _initial_lambdas, psi, _etas = solve(
                all_data,
                input,
                ctx,
                etas,
            )

        except Exception as e:
            print(e)
    raise ValueError("Not implemented")

    if ctx.debug:
        print("run final time")
    deltas, lambdas, psi, etas = solve(
        all_data,
        input,
        ctx,
        etas,
    )
    for i in range(all_data.venues_count):
        print(
            f"Market {all_data.venue(i)}, delta: {deltas[i].value}, lambda: {lambdas[i].value}, eta: {etas[i]}",
        )

    # basically, we have matrix where rows are in tokens (DELTA)
    # columns are outs (LAMBDA)
    # so recursively going DELTA-LAMBDA and subtracting values from cells
    # will allow to build route with amounts
    return (psi.value[all_data.index_of_token(input.out_token_id)], psi)
