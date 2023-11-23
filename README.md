---
tags: "#mev #composable #cvm #mantis #defi #cosmos #cosmwasm #evm #ibc"
---

For `issues`:

https://github.com/ComposableFi/composable/

For `tech question and talks`:

https://discord.com/channels/828751308060098601/1162324949277622333

https://discord.com/channels/828751308060098601/1163404253537247283

For `higher level docs`:

https://github.com/ComposableFi/composable/tree/main/docs/docs/technology/mantis




## Plan for todayS

1. move CVM shared code here, make it build as part of composable as include src
2. build generator of CVM into contract
3. build storage for solution tacking with is own VW
3. add block to order filled
4. put volume into solution
5. split solution contract into algorithms.rs and cw storage deps
6. describe data model of contract
7. finalize porting bruno solver to rust
9. runs bruno simulator to have same output
8. use solver from rust as bruno used in rust
9. query pools of osmosis (using state or JSON? JSON 100%)
10. push ATOM to CVM configs on all chains
11. add support for cvm/ prefixed asses to contract
12. query CVM for mapping of assets
13. in and out data for bruno algortihm
14. fix cross chain movements integration, imrove tracking code
15. teh docs
15. deploy/redeploy/reredeploy with fixes
16. test assets on osmosis
17. NEUTRON CVM adapter
17. NEUTRON CVM devnet test
17. NEUTRON CVM mainnet testP
17. NEUTRON CVM test joe assets
18. tech docs
18. redeplo solver with fixes
19. update final Google docs
20. "publish" bruno paper
21. elimniate what was done from https://github.com/ComposableFi/cvm-old
22. MOVE TS generator HERE
23. Publush from Composable REPO or ENV repo, move JSONs here too