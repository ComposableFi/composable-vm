#!/bin/sh

poetry lock --no-update
poetry install
poetry run ruff format . --config pyproject.toml 
poetry run ruff . --exit-non-zero-on-fix --config pyproject.toml --fix