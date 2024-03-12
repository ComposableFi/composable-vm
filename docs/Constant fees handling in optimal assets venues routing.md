# Constant fees handling in optimal assets heteregoneus routing

User side of problem described well in Composble Foundation documents about intents.

Solution side with based on operational research(OR) described in `Optimal Routing for Constant Function Market Makers`.

Let make layman clarification of section 5 of that paper on solving with constant fees

## Constant fees constraints

```python
objective = maximize(received_amount - trade_decision * constant_fee)
constraint(venue_amount <= trade_decision * maximal_possible_trade)
trade_decision[i] in [0,1]
M = len(trade_decision)
```

Logically if `trade_decision` is `0` for some venue, it does not penalizes objective nor allowed to be traded over.

If it it is equal `1`, than trade up to maximal possible trade is allowed and full constant fees penalized.

## So what is the problem?

Let setup real world scenario here.

Constant fees for transfers can differ by $10^5$ magnitude. 

Numeric values for reservers, tendered and received can vary from $10^3$ to $10^{24}$ in one problem definion. 

Solving OR in integers nor in real numbers is not possible with such numbers.


## So what we do?

We scale all numbers to range of $10^{-4}$ to $10^4$. Scaling procedure is not trivial and may intorduce some error into numbers.

Than we solve with scaled real numbers.

Could we scale to $1$ and $10^5$ to solve in integers right away? Not in practice if we want to solve fast again many venues, which works only for real numbers.

## Lets solve for real.

For great variety to simple cases things works as intended.

But for some important production cases things go wild.

- `trade_decision` is set `1` trading tiny amounts
- `trade_decision` is set to `0` even trading critical venue (the only venue which can trade from tendered to received)
- `venue_amount` are very approximate and do not lead to exact route they induce to do

None of these trivitial to solve. 

For example, what is tiny amount in real (USD value) or in numeric are hard to define and use.

Setting some `trade_decision` for critical path also solveable logically, solver engine fails to find that path, reporting infeacible.

## What paper suggest?

It suggest using integer proramming, but integer progamming cannot be done without integer scaling and elimination of non trade pools. So need to solve real first.

It tells we can sort `trade_decision` for some problem, and solver hopefully can solve linearly $O(M)$ where $M$ is venue count or randomly.

So problm araises when best `trade_decision` taken with large volume, can became bad decision later and good again as we go using our $O(M)$ scan, because fees on and off.

Enumerating all possible `trade_decision` is $O(2^M)$.

On common hardware each run of OR solver takes 1-2 seconds on dozen of venues, so it leads to hundreds of seconds to solve. What would be on hundreds of venue? $O(M)$ is not good, need $O(log \ M)$.

Randomization is not an option as it may miss critical path.

## Real world cases
 
Next table tells to trade over venue if it is multiplied by `trade_decision=1.0`.

So `11*(1.0)` and `12*(1.0)` which are transfers of PICA to Osmosis and OSMO to composable. but were is exchange of PICA to OSMO?

As you can see `0*(-0.0)` which tells not to trade at all. 


This is one of dozen cases tested within current MANTIS Solver Blackbox codebase.


```
0*(-0.0)(10000.0/09<->10000.0/10),delta=[9.99900513e-01 4.83004546e-07],lambda=[4.83103542e-07 9.99700573e-01]
1*(-0.0)(10000.0/10<->9999.999999999998/12),delta=[8.49388192e-07 1.50926511e-06],lambda=[1.50907969e-06 8.49337181e-07]
2*(-0.0)(10000.0/13<->10000.0/10),delta=[6.90021013e-07 9.65741019e-07],lambda=[6.60365208e-07 9.95218129e-07]
3*(-0.0)(10000.0/10<->10000.0/16),delta=[9.73609037e-07 9.87462450e-07],lambda=[9.87363778e-07 9.73509796e-07]
4*(-0.0)(10000.0/11<->10000.0/10),delta=[1.79167031e-06 9.65795946e-07],lambda=[1.76201449e-06 9.95158686e-07]
5*(-0.0)(10000.0/14<->10000.0/10),delta=[8.77539628e-07 1.46569450e-06],lambda=[1.46551313e-06 8.77486806e-07]
6*(-0.0)(10000.0/12<->10000.0/16),delta=[9.78733354e-07 9.82332683e-07],lambda=[9.82234365e-07 9.78634834e-07]
7*(-0.0)(10000.0/10<->9999.999999999998/12),delta=[1.48109859e-06 8.54123119e-07],lambda=[8.54092557e-07 1.48089423e-06]
8*(-0.0)(10000.0/14<->10000.0/10),delta=[1.49153690e-06 8.73919282e-07],lambda=[8.73907587e-07 1.49131377e-06]
9*(-0.0)(10000.0/15<->10000.0/16),delta=[9.84432972e-07 9.76633766e-07],lambda=[9.76534470e-07 9.84333795e-07]
10*(-0.0)(9999.999999999998/10<->9999.999999999998/15),delta=[9.69706603e-07 9.91361456e-07],lambda=[9.91260380e-07 9.69604146e-07]
11*(1.0)(10000.0/09<->10000.0/73),delta=[4.83153060e-07 1.00000099e+00],lambda=[9.99900503e-01 9.75859992e-07]
12*(1.0)(10000.0/10<->10000.0/74),delta=[9.99700758e-01 4.82955531e-07],lambda=[4.82955733e-07 9.99600798e-01]
13*(-0.0)(10000.0/11<->10000.0/75),delta=[9.70607168e-07 9.85438451e-07],lambda=[9.90312515e-07 9.75487977e-07]
14*(-0.0)(10000.0/12<->10000.0/78),delta=[9.70627783e-07 9.85459875e-07],lambda=[9.90333123e-07 9.75509402e-07]
15*(-0.0)(10000.0/13<->10000.0/82),delta=[9.70652978e-07 9.85483967e-07],lambda=[9.90358311e-07 9.75533495e-07]
16*(-0.0)(10000.0/15<->10000.0/81),delta=[9.70628839e-07 9.85460536e-07],lambda=[9.90334179e-07 9.75510063e-07]
17*(0.0)(0/50<->0/77),delta=[0. 0.],lambda=[0. 0.]
18*(-0.0)(10000.0/80<->10000.0/14),delta=[9.85460011e-07 9.70628291e-07],lambda=[9.75509538e-07 9.90333631e-07]
19*(-0.0)(10000.0/47<->10000.0/10),delta=[9.85459337e-07 9.70627452e-07],lambda=[9.75508864e-07 9.90332793e-07]
```


Let solve for ust PICA to OSMO on Osmosis. It solves same `-0.0` for `100/PICA`, for `1000/PICA` or for `10000/PICA` it can loop freeze or tell infeasible, like solving large and small valut but stuck on middle one.

Sometimes `venue_amount` go negative because of rounding. 

Attemps to restrict trading many small amounts by reducing arbitrage hard to define.

```
11*(0.0)(10000.0/09<->10000.0/73),delta=[0. 1.],lambda=[0. 0.]
```

Above traded 10K of PICA agains 100M PICA pools and OSMO, and yet it traded to zero, which is not right.

Various weird artifactes appear in wide range of constraints, from most relaxed to most strict.

Another instability, solved and produced some ETAs, but when forcing exact same, assuming will get same solution. Got infeciable, meaning that forcing what calculated and should be 100% solved, fails.

```
2024-03-02 22:26:29.206 | INFO     | simulation.routers.angeris_cvxpy.data:summary:217 -  raw_in=-1.0->raw_out=0.9996001796662408((1.6090813573421516e-06/USD))
2024-03-02 22:26:29.206 | INFO     | simulation.routers.angeris_cvxpy.data:summary:220 - original_in=1.0000000000000002/158456325028528675187087900673(0.011315484012931223/USD),original_out=1/158456325028528675187087900674
2024-03-02 22:26:29.208 | INFO     | simulation.routers.angeris_cvxpy.data:summary:224 - 0*(1.0)(10000.0/237684487542793012780631851009<->9999.999999999996/237684487542793012780631851010),delta=[9.99900062e-01 9.00100010e-08],lambda=[5.16932098e-08 9.99700160e-01]
2024-03-02 22:26:29.209 | INFO     | simulation.routers.angeris_cvxpy.data:summary:224 - 1*(0.0)(9999.999999999998/237684487542793012780631851010<->10000.0/237684487542793012780631851012),delta=[2.02483123e-07 1.87026614e-07],lambda=[2.73982269e-07 1.15501383e-07]
2024-03-02 22:26:29.210 | INFO     | simulation.routers.angeris_cvxpy.data:summary:224 - 2*(0.0)(10000.0/237684487542793012780631851013<->9999.999999999998/237684487542793012780631851010),delta=[2.57863924e-07 1.44026398e-07],lambda=[2.47873925e-07 1.53989148e-07]
2024-03-02 22:26:29.212 | INFO     | simulation.routers.angeris_cvxpy.data:summary:224 - 3*(0.0)(9999.999999999998/237684487542793012780631851010<->10000.0/237684487542793012780631851016),delta=[1.42094493e-07 1.43689253e-07],lambda=[1.43727874e-07 1.42038824e-07]
2024-03-02 22:26:29.213 | INFO     | simulation.routers.angeris_cvxpy.data:summary:224 - 4*(0.0)(10000.0/237684487542793012780631851011<->9999.999999999996/237684487542793012780631851010),delta=[1.01813728e-07 1.35394300e-07],lambda=[9.18237288e-08 1.45370162e-07]
2024-03-02 22:26:29.215 | INFO     | simulation.routers.angeris_cvxpy.data:summary:224 - 5*(0.0)(10000.0/237684487542793012780631851014<->9999.999999999998/237684487542793012780631851010),delta=[2.10644565e-07 1.76245668e-07],lambda=[2.62631559e-07 1.24232338e-07]
2024-03-02 22:26:29.217 | INFO     | simulation.routers.angeris_cvxpy.data:summary:224 - 6*(0.0)(10000.0/237684487542793012780631851012<->10000.0/237684487542793012780631851016),delta=[1.37389267e-07 1.48622150e-07],lambda=[1.49861348e-07 1.36134380e-07]
2024-03-02 22:26:29.218 | INFO     | simulation.routers.angeris_cvxpy.data:summary:224 - 7*(0.0)(9999.999999999998/237684487542793012780631851010<->9999.999999999998/237684487542793012780631851012),delta=[1.30320549e-07 1.22874123e-07],lambda=[8.12427273e-08 1.71937274e-07]
2024-03-02 22:26:29.220 | INFO     | simulation.routers.angeris_cvxpy.data:summary:224 - 8*(0.0)(10000.0/237684487542793012780631851014<->9999.999999999998/237684487542793012780631851010),delta=[1.38259018e-07 1.18777775e-07],lambda=[7.62820254e-08 1.80743380e-07]
2024-03-02 22:26:29.222 | INFO     | simulation.routers.angeris_cvxpy.data:summary:224 - 9*(0.0)(9999.999999999998/237684487542793012780631851015<->10000.0/237684487542793012780631851016),delta=[1.44667328e-07 1.40929543e-07],lambda=[1.40501785e-07 1.45077743e-07]
2024-03-02 22:26:29.223 | INFO     | simulation.routers.angeris_cvxpy.data:summary:224 - 10*(0.0)(9999.999999999996/237684487542793012780631851010<->10000.0/237684487542793012780631851015),delta=[1.40281283e-07 1.45639082e-07],lambda=[1.46087311e-07 1.39814626e-07]
2024-03-02 22:26:29.224 | INFO     | simulation.routers.angeris_cvxpy.data:summary:224 - 11*(1.0)(10000.0/237684487542793012780631851009<->10000.0/158456325028528675187087900673),delta=[-0.  1.],lambda=[0.9999 0.    ]
2024-03-02 22:26:29.225 | INFO     | simulation.routers.angeris_cvxpy.data:summary:224 - 12*(1.0)(9999.999999999998/237684487542793012780631851010<->9999.999999999998/158456325028528675187087900674),delta=[9.9970023e-01 9.0000010e-08],lambda=[9.0000010e-08 9.9960027e-01]
2024-03-02 22:26:29.226 | INFO     | simulation.routers.angeris_cvxpy.data:summary:224 - 13*(0.0)(10000.0/237684487542793012780631851011<->10000.0/158456325028528675187087900675),delta=[-0. -0.],lambda=[ 0. -0.]
2024-03-02 22:26:29.228 | INFO     | simulation.routers.angeris_cvxpy.data:summary:224 - 14*(0.0)(10000.0/237684487542793012780631851012<->10000.0/158456325028528675187087900678),delta=[-0. -0.],lambda=[ 0. -0.]
2024-03-02 22:26:29.229 | INFO     | simulation.routers.angeris_cvxpy.data:summary:224 - 15*(0.0)(10000.0/237684487542793012780631851013<->10000.0/158456325028528675187087900682),delta=[-0. -0.],lambda=[ 0. -0.]
2024-03-02 22:26:29.230 | INFO     | simulation.routers.angeris_cvxpy.data:summary:224 - 16*(0.0)(10000.0/237684487542793012780631851015<->10000.0/158456325028528675187087900681),delta=[-0. -0.],lambda=[ 0. -0.]
2024-03-02 22:26:29.231 | INFO     | simulation.routers.angeris_cvxpy.data:summary:224 - 17*(0.0)(0/316912650057057350374175801350<->0/158456325028528675187087900677),delta=[0. 0.],lambda=[0. 0.]
2024-03-02 22:26:29.233 | INFO     | simulation.routers.angeris_cvxpy.data:summary:224 - 18*(0.0)(10000.0/158456325028528675187087900680<->10000.0/237684487542793012780631851014),delta=[-0. -0.],lambda=[-0.  0.]
2024-03-02 22:26:29.234 | INFO     | simulation.routers.angeris_cvxpy.data:summary:224 - 19*(0.0)(9999.999999999998/316912650057057350374175801347<->9999.999999999998/237684487542793012780631851010),delta=[-0. -0.],lambda=[-0.  0.]
2024-03-02 22:26:29.515 | INFO     | simulation.routers.generic_linear:solve:38 - using forcing etas [None, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, None, None, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0] for solution
```

All delta and lambdas also approximate, see:
```
2024-03-04 11:01:41.210 | ERROR    | simulation.routers.angeris_cvxpy.data:__init__:42 - 11 [3.92902084e-04 1.00078549e+00] [1.00029273e+00 7.85520222e-04] 1.0 
2024-03-04 11:01:41.210 | ERROR    | simulation.routers.angeris_cvxpy.data:__init__:43 - 11 [392902085, 1000785490944] [1000292725692, 785520222] 1 
```

So noise in delta/lambda increased after scale out, hard to eliminate.

## So what?

It does not look that OR solver works well without sophisticated procedures to define  decision of trade or not. To take decsion and scale, oracles requried. Oracles are build using combinatiorial (graph) optimization. So we should (should have been go that way), also it does not do arbitrage nor splits to many-to-many trade, but it can robostly solve simpl paths.

