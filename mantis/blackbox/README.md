# Readme

Python server which solves problems from provided input and additional data obtained from chains/indexers and outputs single route solution.

Blackbox never calls any transactions, because it is need higher level of security and because TX clients are binary highly tuned to network, to which Python may not hae good access.


## Integration

### Dependencies

All Python facilities (including data mangling) are done in `../simulation` codebase.

`../../crates/cvm` and `../../crates/mantis` provide JSON schemas for relevant data.

Use schemas from Neutron/Skip/Osmosis as needed. 

Code as simples web service most popular in data science people named ???.

### Data

Server uses Rust JSON Schema data types for CVM/MANTIS contracts when available.

Python calls off chain APIs for Osmosis/Neutron about Pools. And SKIP API about relays using their schemas.

No heavy lifting of data. Just simple gets.

This code never simulates transactions, all such input provided inside call.

Blackbox is not interactive regarding transactions.

### Sandbox

All data getting to be here. So that research can get current state of data just immediately to run numerics and debug.

All calls are hidden behind `data` function. Including `../../routing.md` data (about routes possible).

Data-faced maps Cosmos assets and data to CVM/MANTIS data, and mangles it to provide

Server hosted via https://github.com/ComposableFi/env with full access of relevant people. 

Server works in constant restart in case of failure.

Server fails on any overflow or undecided input, so it can be logged and fixed.

