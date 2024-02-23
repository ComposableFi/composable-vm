from typing import TypeVar

TId = TypeVar("TId", int, str)
"""
This is global unique ID for token(asset) or exchange(pool)
"""

TNetworkId = TypeVar("TNetworkId", int, str)
TAmount = TypeVar("TAmount", int, str)
