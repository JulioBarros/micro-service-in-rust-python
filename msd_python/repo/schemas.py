""" Our non database related models/types. """

import datetime
from pydantic import BaseModel


class Instrument(BaseModel):
    symbol: str
    fetched: datetime.datetime


class Quote(BaseModel):
    symbol: str
    date: datetime.datetime
    open: float
    high: float
    low: float
    close: float
    volume: int


class Stats(BaseModel):
    symbol: str
    start: datetime.datetime
    end: datetime.datetime
    count: int
    mean: float
    max: float
    min: float
