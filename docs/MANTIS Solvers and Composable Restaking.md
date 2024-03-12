# Overview

This documents outlines how MANTIS Solver would use Composable Restaking to avoid rug users by providing misleading multi domain(multi block, multi chain, multi transaction) execution of wintents.

Also it reflects on how CVM can be used over IBC to participate in slashing(fisherman+misbehaviour) protocol.


## Restaking

Is:

1. Anystaking, contracts to allow stake/delegates anything(restake)
2. AVS - offchain components to execute integration with offchain protocols
3. Integration of 1 and 2, specifically slashing

## Solver

Finds as set of cross chain routes expressed as CVM on chain program to execute user intent (for example, cross chain transfer and exchange).

MANTIS exagarates problem because users are do not verify routes explicitly, like in SKIP ibc.fun and Paraswap.

MANTIS Solver is AVS. 

## Rug

Multidomain nature of CVM solutions prevent their single block simulation, verification and reward. CVM can rug.

Solution is Stake assigned to Solver which is Slashed in case of bad execution.

## Misbehaviour

That is point 3 in the list.

One way to is having set of validators observe domains and solution.

In case of Solver solution fails to deliver promised settlement, validators 2/3 sign Slashing message of Solver Stake (collateral).

Other way, is for fishermans to pemissionsly crank CVM execution  of MANTIS outposts on each chain were solution to be settled within validated timeframe.

CVM results in multihop propagation of failure prove up to Restaking chain.


## Notes

Likely for various reasons validators 2/3 still will be needed to support MANTIS CVM based flow.


Other operations like `Stake` and `Delegate` to support more Restaking flows are in Composable VM specification.

Regardeless of solution, Validators and Indexers need unified realtime view of multi domain data to detect misbihaviours.