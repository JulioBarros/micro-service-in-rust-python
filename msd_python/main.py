"""Main entry point for FastAPI based Python web/micro service. """

from datetime import timedelta

from fastapi import Depends, FastAPI
from sqlalchemy.orm import Session

from repo import crud, models, util
from repo.database import SessionLocal, engine

models.Base.metadata.create_all(bind=engine)

app = FastAPI()


def get_db():
    """ Helper function / generator to get database connection. 
        Injected into route handlers by FastAPI framework. """

    try:
        db = SessionLocal()
        yield db
    finally:
        db.close()


@app.get("/")
async def system_summary(db: Session = Depends(get_db)):
    """ Used to check if the service and databaseis up. 
        Does a basic query and returns the instruments
        we've cached so far. """

    instruments = crud.get_instruments(db)
    return {"instruments": instruments}


@app.get("/instruments/{symbol}")
async def get_quotes(
    symbol: str,
    start: str = None,
    end: str = None,
    duration: int = None,
    db: Session = Depends(get_db),
):
    """ Get quotes (open, high, low, close) for a given instrument for a
        given time period.  The period can be set by specifing the start
        and end dates or the start or end and duration. If nothing is specified
        the current date is taken as the end date and a default duration of
        10 days is used. """

    start_date, end_date, duration = util.parse_params(start, end, duration)
    quotes = crud.get_quotes(db, symbol, start_date, end_date)

    return {
        "symbol": symbol,
        "start": start_date,
        "end": end_date,
        "duration": duration,
        "quotes": quotes,
    }


@app.get("/instruments/{symbol}/stats")
async def get_status(
    symbol: str,
    start: str = None,
    end: str = None,
    duration: int = None,
    db: Session = Depends(get_db),
):
    """ Calculate the stats for the specified symbol over the time period
        specified (see /instruments/{symbol}). Stats include the count, mean,
        min, max for the symbol during the period. """

    start_date, end_date, duration = util.parse_params(start, end, duration)
    stats = crud.get_stats(db, symbol, start_date, end_date)
    stats["duration"] = duration

    return stats


@app.get("/instruments/{symbol}/crossovers")
async def get_crossovers(
    symbol: str,
    start: str = None,
    end: str = None,
    duration: int = None,
    small: int = 5,
    large: int = 10,
    db: Session = Depends(get_db),
):
    """ Extra route to provide a little more functionality. """

    first, last, duration = util.parse_params(start, end, duration)

    small_stats = []
    large_stats = []
    for offset in range(0, (last - first).days):
        end_date = first + timedelta(days=offset)
        sm_delta = timedelta(days=small)
        lg_delta = timedelta(days=large)
        small_stats.append(crud.get_stats(db, symbol, end_date - sm_delta, end_date))
        large_stats.append(crud.get_stats(db, symbol, end_date - lg_delta, end_date))

    for sm, lg in zip(small_stats, large_stats):
        assert sm["end"] == lg["end"]

    signals = [
        {
            "date": sm["end"],
            "long": (sm["mean"] > lg["mean"]),
            "short": (sm["mean"] < lg["mean"]),
            f"{small:02}sma": sm["mean"],
            f"{large:02}sma": lg["mean"],
        }
        for sm, lg in zip(small_stats, large_stats)
    ]

    crossovers = []
    long_signal = False
    short_signal = False
    for s in signals:
        if s["long"] != long_signal or s["short"] != short_signal:
            quote = crud.get_quote_as_of(db, symbol, s["date"])
            s["close"] = quote.close
            crossovers.append(s)
            long_signal = s["long"]
            short_signal = s["short"]

    return crossovers
