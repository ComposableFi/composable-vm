# for alignment on input and output of algorithm
from functools import cache
from pathlib import Path
import pandas as pd
from enum import Enum
from typing import TypeVar, Generic
from pydantic import BaseModel, validator
from strictly_typed_pandas import DataSet

from mantis.simulation.routers.data import AssetPairsXyk, AssetTransfers, read_dummy_data

TEST_DATA_DIR = Path(__file__).resolve().parent / "data"

def test_all_data_from_csv():
    assert(read_dummy_data(TEST_DATA_DIR))