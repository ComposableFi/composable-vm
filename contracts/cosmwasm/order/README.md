# Overview

This contract implements `Order` instruction of CVM via (CoW)(https://en.wikipedia.org/wiki/Coincidence_of_wants).
It integrates CoW with CFMM cross chain execution found by Solvers.

## General flow

Some users request exchange some assets to other assets.
Theirs tokens were transferred via well known bridges to here. 

User sends transactions containing order, describing amounts of assets they have(give) and want(take).
Additionally they describe if they allow partial fill and timeout.
Both assets must be local and registered in CVM.
If target wanted out asset is just bridged, transfer path must be provided. 

Solvers read all user orders on chain, and propose solutions to do CoW amid orders (including order from solver which may come with solution),
and cross chain CFMM for the rest. 
Each solver account has one and only one solution per pair. So solver can send and resend solution to replace old one for specific pair.

Solution with maximum volume is accepted, and should settle on second largest volume (second bid auction). 

Bidders reserve amounts as rewards to pay for percentage of volume solved via their preferred CFMM.

More details semantics will be described in whitepaper. including Solver operations.

## Solution

In order solution have to be accepted, it must:

- all CVM programs has salt to be solution id from hash of solver address, token a denom, token b denom and block solution was added
- transfer all amounts as absolute as CVM instruction for cross chain part
- all COWs solved against same price
- none of user limits violated
- only assets from orders inside solution are used
- CVM must adhere Connection.fork_join_supported, for now it is always false (it restrict set routes possible)
- Must adhere to single assets per spawn if multi assets are not supported (all for now)

CoW cannot violate user limit, even if violating user limit allows to catch up better with CVM route.

## Implementation

Current implementation is for 1 solution for one user, 1 to 1 asset, permissioned solvers without collateral.

### Bidding

Bidder call this contract with CFMM identifier and percentage of volume they bid. 
This contract instantiates CVM executor per CFMM and transfer amount to it, and tracks percentage per bidder.
When bidder joins, percentage averaged appropriately, with always larger bid for same volume.
For each accepted route, Solver is delegated with relevant amount to withdraw.

## Data model

Order has numeric incremental identifier per pair of tokens.

Solution has owner per pair as identifier.

If solution requires multiblock tracking for execution its id added with block solution was accepted.



## References

https://github.com/openbook-dex/openbook-v2/blob/master/programs/openbook-v2/src/lib.rs