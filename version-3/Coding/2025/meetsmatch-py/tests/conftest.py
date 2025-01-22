import os
import sys

# Add project root to Python path FIRST
sys.path.insert(0, os.path.abspath(os.path.join(os.path.dirname(__file__), "..")))

import pytest
from sqlalchemy import create_engine, event
from sqlalchemy.orm import sessionmaker, scoped_session
from src.meetsmatch.models import Base

# Configure test database 
TEST_DATABASE_URL = "sqlite:///./test_meetsmatch.db?check_same_thread=False"

@pytest.fixture(scope="session")
def engine():
    engine = create_engine(TEST_DATABASE_URL)
    # Create fresh tables at start of test session
    Base.metadata.drop_all(engine)
    Base.metadata.create_all(engine)
    return engine

@pytest.fixture(scope="function")
def connection(engine):
    """Connection fixture with nested transaction"""
    connection = engine.connect()
    transaction = connection.begin_nested()
    yield connection
    transaction.rollback()
    connection.close()

@pytest.fixture(scope="function")
def db_session(connection):
    """Session fixture bound to connection with nested transaction"""
    Session = scoped_session(sessionmaker(bind=connection))
    session = Session()
    session.begin_nested()

    @event.listens_for(session, "after_transaction_end")
    def restart_savepoint(session, transaction):
        if transaction.nested and not transaction._parent.nested:
            session.begin_nested()

    yield session
    session.rollback()  # Explicit rollback
    session.close()

@pytest.fixture(autouse=True)
def reset_mocks(mocker):
    """Reset all mocks after each test"""
    yield  # Let the test run first
    mocker.resetall()
