

# How indexer works

It takes purely on chain data from registry from starter chain.

Than it joins it with offchain static data about chains as defined in VCS repo.

Than it gets data from available source about recent history summaries and current snapshots from running 3rd party systems.

All that data is used to build oracles.

Finally data is converted to simulator compatible data.