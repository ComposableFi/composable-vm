# How to contribute solver

## 1. Files

Solver to be separate file.
There should be test prefixed python file in same folder as solver simulating and/or testing case(s) of data input

## 2. Installable

In case solver can be easy installed locally (needs registration to download and cannot distiribute downloaded code),
NLP solvers has servers like NEOS, Gurobi, MOSEK and standard format to consume
Any solver Python code used to call it to be installed via ../../pyproject.toml Poetry dependency

## 3. Proven

There must be few tests and documentation.

If there are a lot of simulation/tests, docs are less needed.

In case of few tests, need to write next.

Solver type(NLP, MILP, etc) to be described.
Difference from convex optimal routing solver to be described.

## 4. Production

Should use ../data.py as input/output.
