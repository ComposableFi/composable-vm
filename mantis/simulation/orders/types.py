from enum import Enum


class OrderType(Enum.Enum):
    BUY = "Buy"
    SELL = "Sell"

    def __str__(self) -> str:
        return self.value
