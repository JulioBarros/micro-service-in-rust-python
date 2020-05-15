""" Module with functions that directly interface with the database. """

from datetime import datetime

from sqlalchemy import func
from sqlalchemy.sql import label
from sqlalchemy.orm import Session
import requests

from repo import models, util

# This is the response format we get from the service.
# {
#     "symbol": "AAPL",
#     "historical": [
#         {
#             "date": "2015-05-04",
#             "open": 129.5,
#             "high": 130.57,
#             "low": 128.26,
#             "close": 128.7,
#             "adjClose": 118.44,
#             "volume": 5.09883e7,
#             "unadjustedVolume": 5.09883e7,
#             "change": -0.8,
#             "changePercent": -0.618,
#             "vwap": 129.17667,
#             "label": "May 04, 15",
#             "changeOverTime": -0.00618,
#         }, ...
#     ],
# }


def historical_to_quote(symbol, h: any) -> models.Quote:
    """ Translates the service record to our db type. """
    d = {
        "symbol": symbol,
        "date": util.parse_date(h["date"]),
        "open": h["open"],
        "high": h["high"],
        "low": h["low"],
        "close": h["adjClose"],
        "volume": h["unadjustedVolume"],
    }
    return models.Quote(**d)


def fetch_if_necessary(db: Session, symbol: str):
    """ Updates the given symbol daily incase there are any adjustments.
        Price adjustments can occur without notice so we have to fetch
        on a daily basis for now. """

    need_fetch = False

    last_fetch = (
        db.query(models.Instrument).filter(models.Instrument.symbol == symbol).first()
    )

    # If the symbol is not in the database or it was fetched more than a day
    # ago, we need to fetch it.
    if not last_fetch:
        last_fetch = models.Instrument(symbol=symbol)

    if not last_fetch or not last_fetch.fetched or last_fetch.fetched < util.today():
        need_fetch = True

    # If we need to fetch it, call the service, delete the old records and insert the new ones.
    if need_fetch:
        # Do the fetch
        url = f"https://financialmodelingprep.com/api/v3/historical-price-full/{symbol.upper()}"
        r = requests.get(url)
        data = r.json()

        # Delete old records
        db.query(models.Quote).filter(models.Quote.symbol == symbol).delete()

        # Convert and insert new records
        for q in data["historical"]:
            quote = historical_to_quote(symbol, q)
            db.add(quote)

        # Update the symbol and commit
        last_fetch.fetched = datetime.now()
        db.add(last_fetch)
        db.commit()


def get_instruments(db: Session):
    """ Get a list of all the instruments in the db. """

    return db.query(models.Instrument).all()


def get_quotes(db: Session, symbol: str, start: datetime, end: datetime):
    """ Get quotes for a specific symbol. """

    fetch_if_necessary(db, symbol)
    return (
        db.query(models.Quote)
        .filter(models.Quote.symbol == symbol)
        .filter(models.Quote.date >= start)
        .filter(models.Quote.date <= end)
        .all()
    )


def get_quote_as_of(db: Session, symbol: str, date: datetime):
    """ Get quotes for a symbol as of a given date. Date could
        be over weekend or holiday so use the previous date that
        we have information for. """

    fetch_if_necessary(db, symbol)
    return (
        db.query(models.Quote)
        .filter(models.Quote.symbol == symbol)
        .filter(models.Quote.date <= date)
        .order_by(models.Quote.date.desc())
        .first()
    )


def get_stats(db: Session, symbol: str, start: datetime, end: datetime):
    """ Calculate some basic stats for the given symbol during the given
        date time range. Note I decided to compute some of the stats in
        the database. I don't think the db has all the numerical functions
        we'd want IRL so we may have to compute them in code. """

    fetch_if_necessary(db, symbol)
    stats = (
        db.query(
            func.count(models.Quote.date).label("count"),
            func.avg(models.Quote.close).label("mean"),
            func.min(models.Quote.close).label("min"),
            func.max(models.Quote.close).label("max"),
        )
        .filter(models.Quote.symbol == symbol)
        .filter(models.Quote.date >= start)
        .filter(models.Quote.date <= end)
        .first()
    )
    return {
        "symbol": symbol,
        "start": start,
        "end": end,
        "count": stats.count,
        "mean": stats.mean,
        "min": stats.min,
        "max": stats.max,
    }
