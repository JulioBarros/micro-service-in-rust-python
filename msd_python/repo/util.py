""" Some utilities, mostly to simplify date time handling and parsing. """

from typing import Union, Optional, Tuple
from datetime import datetime, timedelta

import dateutil.parser

DEFAULT_DURATION = 14


def parse_date_or_default(base_date: Union[datetime, str, None], offset: int = 0):
    """ If there is a date string parse it else create one and return it plus (or minus)
        an offset. """

    if base_date is None:
        base_date = today()
    if isinstance(base_date, str):
        base_date = dateutil.parser.parse(base_date)

    return base_date + timedelta(days=offset)


def parse_date(ds: str):
    """ Parse a date string. """

    return dateutil.parser.parse(ds)


def today():
    """ Create a datetime as of midnight today / this morning. """

    dt = datetime.now()
    return datetime(dt.year, dt.month, dt.day)


def parse_params(
    start: Optional[str], end: Optional[str], duration: Optional[int]
) -> Tuple[datetime, datetime, int]:
    """ Brute force processing of our start, end, duration parameter
        rules. Basically, we need to get to a date range and you can 
        specify that start and end, the start or end and a duration, 
        or we calculate the range based on today and a duration (using
        a default duration if necessary. """

    # 1 1 1
    if start and end and duration:
        end_date = parse_date_or_default(end)
        start_date = parse_date_or_default(start)
        if duration != (start_date - end_date).days:
            print("ERROR: ", start, end, duration)
    # 1 1 0
    elif start and end and not duration:
        end_date = parse_date_or_default(end)
        start_date = parse_date_or_default(start)
        duration = (start_date - end_date).days
    # 1 0 1
    elif start and not end and duration:
        start_date = parse_date_or_default(start)
        end_date = parse_date_or_default(start, duration)
    # 1 0 0
    elif start and not end and not duration:
        duration = DEFAULT_DURATION
        start_date = parse_date_or_default(start)
        end_date = parse_date_or_default(start_date, duration)
    # 0 1 1
    elif not start and end and duration:
        end_date = parse_date_or_default(end, duration)
        start_date = parse_date_or_default(end_date, -duration)
    # 0 1 0
    elif not start and end and not duration:
        duration = DEFAULT_DURATION
        end_date = parse_date_or_default(end)
        start_date = parse_date_or_default(end_date, -duration)
    # 0 0 1
    elif not start and not end and duration:
        end_date = parse_date_or_default(None)
        start_date = parse_date_or_default(end_date, -duration)
    # 0 0 0
    elif not start and not end and not duration:
        duration = DEFAULT_DURATION
        end_date = parse_date_or_default(None)
        start_date = parse_date_or_default(end_date, -duration)
    else:
        # We really shouldn't get here but for completeness ...
        print("Unexpected state.")

    return start_date, end_date, duration
