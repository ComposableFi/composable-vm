#!/bin/sh

export RUSTFMT=$(which rustfmt)
cargo progenitor --input=../../schema/mantis_solver_blackbox.json --output=src/mantis_solver_blackbox --name=mantis_solver_blackbox --version="3.0.3"