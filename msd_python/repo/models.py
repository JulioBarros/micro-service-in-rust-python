""" Our database related models. """

from sqlalchemy import Column, ForeignKey, String, Integer, Float, DateTime
from sqlalchemy.orm import relationship

from repo.database import Base


class Instrument(Base):
    __tablename__ = "instruments"

    symbol = Column(String, primary_key=True, index=True)
    fetched = Column(DateTime)

    quotes = relationship("Quote", back_populates="instrument")


class Quote(Base):
    __tablename__ = "quotes"

    symbol = Column(
        String, ForeignKey("instruments.symbol"), primary_key=True, index=True
    )
    date = Column(DateTime, primary_key=True, index=True)
    open = Column(Float)
    high = Column(Float)
    low = Column(Float)
    close = Column(Float)
    volume = Column(Integer)

    instrument = relationship("Instrument", back_populates="quotes")
