#!/bin/sh
poetry run pytest -s
poetry run ruff check . --exit-non-zero-on-fix --unsafe-fixes --config pyproject.toml
poetry build -vvv
poetry run blackbox/ -vvv &
sleep 5; curl -v -X 'GET' 'http://0.0.0.0:8000/status' -H 'accept: application/json'
pkill blackbox
poetry check --lock