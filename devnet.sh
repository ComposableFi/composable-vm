#!/bin/sh
#clear && cargo run --bin mantis -- --centauri "http://localhost:26657" --osmosis "localhost:36657" --neutron "localhost:46657" --cvm-contract "centauri1" --wallet "mnemonic" --order-contract "centauri1"


RUST_TRACE=trace cargo run --bin mantis -- --rpc-centauri "https://composable-rpc.polkachu.com:443" --grpc-centauri "https://composable-grpc.polkachu.com:22290" --osmosis "todo" --neutron "todo" --cvm-contract "centauri1wpf2szs4uazej8pe7g8vlck34u24cvxx7ys0esfq6tuw8yxygzuqpjsn0d" --wallet "$WALLET" --order-contract "centauri1lnyecncq9akyk8nk0qlppgrq6yxktr68483ahryn457x9ap4ty2sthjcyt" --simulate "1000000ppica,1ibc/EF48E6B1A1A19F47ECAEA62F5670C37C0580E86A9E88498B7E393EB6F49F33C0"   