#!/bin/sh
#clear && cargo run --bin mantis -- --centauri "http://localhost:26657" --osmosis "localhost:36657" --neutron "localhost:46657" --cvm-contract "centauri1" --wallet "mnemonic" --order-contract "centauri1"


RUST_TRACE=trace cargo run --bin mantis -- --rpc-centauri "https://composable-rpc.polkachu.com:443" --grpc-centauri "https://composable-grpc.polkachu.com:22290" --osmosis "todo" --neutron "todo" --cvm-contract "centauri1wpf2szs4uazej8pe7g8vlck34u24cvxx7ys0esfq6tuw8yxygzuqpjsn0d" --wallet "$WALLET" --order-contract "centauri1lnyecncq9akyk8nk0qlppgrq6yxktr68483ahryn457x9ap4ty2sthjcyt" --simulate "100ppica,1ibc/47BD209179859CDE4A2806763D7189B6E6FE13A17880FE2B42DE1E6C1E329E23"   