# MANTIS CDA security via Composable Restaking

MANTIS CDA has several roles:

- Searchers (end user)
- Sequencer
- Blockbuilder
- Relayer

Composable Restaking has:
- Staker (end user)
- Validator
- Fisherman
- Relayer

Relayer in both cases is same role.


In this document is described some incetives and targets of MANTIS CDA roles,
and how these can be aligned using Composable Restaking.

TLRD; MANTIS CDA roles use Composable Restaking as slashable collateral and reputation system, and infrastucture to achieve both happen.


## MANTIS CDA Roles

### Searcher

He finds arbitrage/MEV in some cross chain activity. He asks Sequencer to commit to execute cross chain what Searcher found, without leaking found opprotunity nor using for own profit.

### Sequencer

Sequencer receives all found opportunites by Searchers and builds cross chain block.

Sequencer asks Blockbuilders to commit for execution for parts of cross chain block.

### Relayer

Sequencer asks Relayer to commit relaying some assets at specific order and specific block.

## Mechanics

## Slashing

Sequencer slashd for failing Searcher execution, assuming he executed on hist own or just failed commitment.

Blockbuilder/Relayer fail their SLA.

## Reputation

MANTIS CDA roles allow to execute higher volume or more oftent if they have higher Stake delegated.

When they slashed, Stakers redelegate stake to other entities.

## Composable Restaking Roles

## Fisherman

Observe failures and reports them to Composable Restaking to Slash to MANTIS CDA Roles.

Observer success and report it to Composable Restaking to unlock Rewards to MANTIS CDA Roles.

## Validators

Verify Fisherman observation and slash/reward accordingly.

## What exactly Fisherman observes/reports and Validators verify?

That to be defined in other document.