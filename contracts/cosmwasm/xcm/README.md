# Testbed to support XCM on Cosmos

`executor` - corresponds to parity `xcm-executor`
`configuraiton` - similar to usual configuration in substrate chain. Maps addresses, IBC connection/ports to and from XCM MultiLocation, barrier
`transactor` - uses `xcm-configuration` and `cw20-ics20` to transact assets
`trap-claim` - lost assets traps and claims
`runtime` - governance of all contracts (like treasury) 
`simulator` helper tools to run `xcm` message amid several contracts on top of `cosmwasm-vm-test`. For each case there will be at least 1 positive test.
`sender` - sends message to IBC
