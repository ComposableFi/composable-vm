# How to contribute solver

1. Solver should consume as input and produce as output shapes(data types) from ../data.py
2. Solver type(NLP, MILP, etc) to be described.
2.1. Difference from convex optimal routing solver to be described.
3. In case solver can be easy installed locally (needs registration to download and cannot distiribute downloaded code),
3.1. NLP solvers has servers like NEOS, Gurobi, MOSEK and standard format to consume
5. Any solver Python code used to call it to be installed via ../../pyproject.toml Poetry dependency
6. There should be test prefixed python file in same folder as solver simulating and/or testing case(s) of data input
