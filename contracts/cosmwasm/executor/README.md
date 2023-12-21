# CVM Executor

Receives and stores user funds.
Fully owned by user.
Delegates cross chain execution to `outpost`.

Instantiated as many instances of the CVM interpreter contract. On some chains, we can use probabilistically generated sub_accounts, but for most, we instantiate a contract instance.

## Events

Note that these events will be yield from the router in production.

### Instantiate contract
```json
{
	"type": "wasm-cvm.executor.instantiated",
	"attributes": [
		{
			"key": "data",
			"value": "{BASE64_ENCODED_DATA}"
		}
	]
}
```

- **BASE64_ENCODED_DATA**: base64 encoded `(network_id, user_id)` pair.

### Execute contract
```json
{
	"type": "wasm-cvm.executor.executed",
	"attributes": [
		{
			"key": "program",
			"value": "{CVM_PROGRAM_TAG}"
		}
	]
}
```

- **CVM_PROGRAM_TAG**: Tag of the executed CVM program

### Execute spawn instruction

```json
{
	"type": "wasm-cvm.executor.spawn",
	"attributes": [
		{
			"key": "origin_network_id",
			"value": "{ORIGIN_NETWORK_ID}"
		},
		{
			"key": "origin_user_id",
			"value": "{ORIGIN_USER_ID}"
		},
		{
			"key": "program",
			"value": "{CVM_PROGRAM}"
		}
	]
}
```

- **ORIGIN_NETWORK_ID**: Network id of the origin. Eg. Picasso, Ethereum
- **ORIGIN_USER_ID**: Chain agnostic user identifier of the origin. Eg. contract_address in Juno
- **CVM_PROGRAM**: Json-encoded cvm program. Note that although it is json-encoded, it is put as a string because of the restrictions of cosmwasm.

## Usage

The CVM interpreter contract interprets the CVM programs. Available instructions are:


### Call
Which is used to call a contract. See that the encoded payload must be in a format:
```
{
	"address": "contract-addr",
	"payload": "json-encoded ExecuteMsg struct"
}
```

### Transfer
Queries `outpost`, gets the contract address and then executes that contract to do the transfer.

### Spawn
Emits `spawn` event with the given parameters.
