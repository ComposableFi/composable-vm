#!/bin/sh
poetry run pytest
poetry run ruff check . --exit-non-zero-on-fix --fix-only --no-unsafe-fixes
poetry check --lock