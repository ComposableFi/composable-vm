poetry lock --no-update
poetry install
poetry run ruff format .
poetry run ruff . --exit-non-zero-on-fix --fix-only --no-unsafe-fixes